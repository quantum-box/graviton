use crate::store::DataStore;
use anyhow::Context;
use chrono::{Duration, NaiveDateTime};
use futures_util::stream::{self, StreamExt};
use std::{collections::{HashMap, HashSet}, io::{Cursor, Read}, sync::Arc};
use serde_json::{json, Value};
use zip::ZipArchive;

use super::{geopolitics, liveuamap};

pub async fn fetch(store: &Arc<DataStore>) -> anyhow::Result<()> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    store.set("gdelt", json!(fetch_gdelt(&client).await.unwrap_or_default()));
    store.set(
        "frontlines",
        geopolitics::fetch_frontlines(&client)
            .await
            .unwrap_or_else(|_| json!({"type":"FeatureCollection","features":[]})),
    );
    store.set(
        "liveuamap",
        json!(liveuamap::fetch_liveuamap(&client).await.unwrap_or_default()),
    );
    store.update_etag("slow");
    Ok(())
}

async fn fetch_gdelt(client: &reqwest::Client) -> anyhow::Result<Vec<Value>> {
    let resp = client
        .get("http://data.gdeltproject.org/gdeltv2/lastupdate.txt")
        .header("User-Agent", "Graviton/1.0")
        .send()
        .await?;
    if !resp.status().is_success() {
        return Ok(Vec::new());
    }

    let last_update = resp.text().await?;
    let latest_url = last_update
        .lines()
        .find_map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            parts.get(2).copied().filter(|url| url.ends_with(".export.CSV.zip"))
        })
        .context("missing export url in lastupdate.txt")?;

    let ts = latest_url
        .rsplit('/')
        .next()
        .and_then(|value| value.strip_suffix(".export.CSV.zip"))
        .context("missing export timestamp")?;
    let latest_ts = NaiveDateTime::parse_from_str(ts, "%Y%m%d%H%M%S")?;

    let urls: Vec<String> = (0..16)
        .map(|offset| {
            let stamp = latest_ts - Duration::minutes(15 * offset);
            format!(
                "http://data.gdeltproject.org/gdeltv2/{}.export.CSV.zip",
                stamp.format("%Y%m%d%H%M%S")
            )
        })
        .collect();

    let responses: Vec<Vec<u8>> = stream::iter(urls.into_iter().map(|url| async move {
        match client.get(&url).header("User-Agent", "Graviton/1.0").send().await {
            Ok(resp) if resp.status().is_success() => resp.bytes().await.ok().map(|bytes| bytes.to_vec()),
            _ => None,
        }
    }))
    .buffer_unordered(6)
    .filter_map(async move |item| item)
    .collect()
    .await;

    let mut events = Vec::new();
    let mut index = HashMap::new();
    let conflict_codes = ["14", "17", "18", "19", "20"];

    for zip_bytes in responses {
        parse_gdelt_zip(&zip_bytes, &conflict_codes, &mut index, &mut events)?;
    }
    Ok(events)
}

fn parse_gdelt_zip(
    zip_bytes: &[u8],
    conflict_codes: &[&str],
    index: &mut HashMap<String, usize>,
    events: &mut Vec<Value>,
) -> anyhow::Result<()> {
    let reader = Cursor::new(zip_bytes);
    let mut archive = ZipArchive::new(reader)?;
    if archive.is_empty() {
        return Ok(());
    }
    let mut file = archive.by_index(0)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let mut csv_reader = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .has_headers(false)
        .flexible(true)
        .from_reader(contents.as_bytes());

    for result in csv_reader.records() {
        let record = match result {
            Ok(value) => value,
            Err(_) => continue,
        };
        if record.len() < 61 {
            continue;
        }
        let event_code = record.get(26).unwrap_or_default();
        let code_prefix = &event_code[..event_code.len().min(2)];
        if !conflict_codes.contains(&code_prefix) {
            continue;
        }

        let lat = match record.get(56).and_then(|value| value.parse::<f64>().ok()) {
            Some(value) if value != 0.0 => value,
            _ => continue,
        };
        let lng = match record.get(57).and_then(|value| value.parse::<f64>().ok()) {
            Some(value) if value != 0.0 => value,
            _ => continue,
        };

        let location = record.get(52).unwrap_or("Unknown").trim();
        let actor1 = record.get(6).unwrap_or("").trim();
        let actor2 = record.get(16).unwrap_or("").trim();
        let source_url = record.get(60).unwrap_or("").trim();
        let date = record.get(1).unwrap_or("").trim();
        let loc_key = format!("{:.1}_{:.1}", lat, lng);

        if let Some(existing) = index.get(&loc_key).copied() {
            let urls = events[existing]["_urls"]
                .as_array_mut()
                .expect("internal gdelt urls array");
            let seen = urls
                .iter()
                .filter_map(Value::as_str)
                .collect::<HashSet<_>>();
            if !source_url.is_empty() && !seen.contains(source_url) && urls.len() < 8 {
                urls.push(json!(source_url));
            }
            let mentions = events[existing]["num_mentions"].as_u64().unwrap_or(1) + 1;
            events[existing]["num_mentions"] = json!(mentions);
            continue;
        }

        let title = if !location.is_empty() && location != "Unknown" {
            location.to_string()
        } else if !actor1.is_empty() || !actor2.is_empty() {
            format!("{} {}", actor1, actor2).trim().to_string()
        } else {
            "Conflict event".to_string()
        };

        events.push(json!({
            "lat": lat,
            "lng": lng,
            "title": title,
            "description": if actor1.is_empty() && actor2.is_empty() {
                Value::Null
            } else {
                json!(format!("{} {}", actor1, actor2).trim())
            },
            "source_url": if source_url.is_empty() { Value::Null } else { json!(source_url) },
            "date": date,
            "num_mentions": 1,
            "source": "GDELT",
            "_urls": if source_url.is_empty() { json!([]) } else { json!([source_url]) },
        }));
        index.insert(loc_key, events.len() - 1);
    }

    for event in events.iter_mut() {
        if event.get("source_url").is_none() || event["source_url"].is_null() {
            if let Some(first_url) = event
                .get("_urls")
                .and_then(Value::as_array)
                .and_then(|urls| urls.first())
                .and_then(Value::as_str)
            {
                event["source_url"] = json!(first_url);
            }
        }
    }

    Ok(())
}
