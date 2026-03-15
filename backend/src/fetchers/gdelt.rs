use crate::store::DataStore;
use serde_json::{json, Value};
use std::sync::Arc;

pub async fn fetch(store: &Arc<DataStore>) -> anyhow::Result<()> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    store.set("gdelt", json!(fetch_gdelt(&client).await.unwrap_or_default()));
    store.set("frontlines", fetch_frontlines(&client).await.unwrap_or_else(|_| json!({"type":"FeatureCollection","features":[]})));
    store.set("liveuamap", json!(fetch_liveuamap(&client).await.unwrap_or_default()));
    store.update_etag("slow");
    Ok(())
}

async fn fetch_gdelt(client: &reqwest::Client) -> anyhow::Result<Vec<Value>> {
    let resp = client
        .get("https://api.gdeltproject.org/api/v2/geo/geo?query=conflict%20OR%20attack%20OR%20military&mode=pointdata&format=geojson&timespan=1d&maxpoints=500")
        .send()
        .await?;
    if !resp.status().is_success() {
        return Ok(Vec::new());
    }
    let data: Value = resp.json().await?;
    let mut events = Vec::new();
    if let Some(features) = data.get("features").and_then(Value::as_array) {
        for f in features {
            let props = f.get("properties").unwrap_or(&Value::Null);
            let geom = f.get("geometry").unwrap_or(&Value::Null);
            let coords = geom.get("coordinates").and_then(Value::as_array);
            if let Some(coords) = coords {
                events.push(json!({
                    "lat": coords.first().and_then(Value::as_f64).map(|_| coords.get(1).and_then(Value::as_f64).unwrap_or(0.0)).unwrap_or(0.0),
                    "lng": coords.first().and_then(Value::as_f64).unwrap_or(0.0),
                    "title": props.get("name").or_else(|| props.get("html")).cloned().unwrap_or(Value::Null),
                    "description": props.get("name").or_else(|| props.get("html")).cloned().unwrap_or(Value::Null),
                    "source_url": props.get("url").cloned().unwrap_or(Value::Null),
                    "num_mentions": props.get("count").and_then(Value::as_u64).unwrap_or(1),
                    "date": props.get("date").cloned().unwrap_or(Value::Null),
                }));
            }
        }
    }
    Ok(events)
}

async fn fetch_frontlines(client: &reqwest::Client) -> anyhow::Result<Value> {
    let tree_resp = client
        .get("https://api.github.com/repos/cyterat/deepstate-map-data/git/trees/main?recursive=1")
        .header("User-Agent", "Graviton/1.0")
        .send()
        .await?;
    if !tree_resp.status().is_success() {
        return Ok(json!({"type":"FeatureCollection","features":[]}));
    }
    let tree: Value = tree_resp.json().await?;
    let latest = tree
        .get("tree")
        .and_then(Value::as_array)
        .and_then(|items| {
            items
                .iter()
                .filter_map(|item| item.get("path").and_then(Value::as_str))
                .filter(|path| path.starts_with("data/deepstatemap_data_") && path.ends_with(".geojson"))
                .max()
                .map(str::to_string)
        });

    let Some(path) = latest else {
        return Ok(json!({"type":"FeatureCollection","features":[]}));
    };
    let resp = client
        .get(format!("https://raw.githubusercontent.com/cyterat/deepstate-map-data/main/{}", path))
        .send()
        .await?;
    if !resp.status().is_success() {
        return Ok(json!({"type":"FeatureCollection","features":[]}));
    }
    let mut geojson: Value = resp.json().await?;
    if let Some(features) = geojson.get_mut("features").and_then(Value::as_array_mut) {
        for (idx, feature) in features.iter_mut().enumerate() {
            if feature.get("properties").is_none() || feature["properties"].is_null() {
                feature["properties"] = json!({});
            }
            feature["properties"]["name"] = json!(match idx {
                0 | 3 => "Russian-occupied areas",
                1 => "Russian advance",
                2 => "Liberated area",
                4 => "Directions of UA attacks",
                _ => "Frontline zone",
            });
            feature["properties"]["zone_id"] = json!(idx);
        }
    }
    Ok(geojson)
}

async fn fetch_liveuamap(client: &reqwest::Client) -> anyhow::Result<Vec<Value>> {
    let mut out = Vec::new();
    for (region, url) in [
        ("Ukraine", "https://liveuamap.com"),
        ("Middle East", "https://mideast.liveuamap.com"),
        ("Israel-Palestine", "https://israelpalestine.liveuamap.com"),
        ("Syria", "https://syria.liveuamap.com"),
    ] {
        let Ok(resp) = client.get(url).header("User-Agent", "Mozilla/5.0").send().await else {
            continue;
        };
        let Ok(body) = resp.text().await else {
            continue;
        };
        if let Some(markers) = extract_ovens_json(&body) {
            for marker in markers.into_iter().take(200) {
                out.push(json!({
                    "id": marker.get("id").cloned().unwrap_or(Value::Null),
                    "title": marker.get("s").or_else(|| marker.get("title")).cloned().unwrap_or(Value::Null),
                    "lat": marker.get("lat").cloned().unwrap_or(Value::Null),
                    "lng": marker.get("lng").cloned().unwrap_or(Value::Null),
                    "timestamp": marker.get("time").cloned().unwrap_or(Value::Null),
                    "link": marker.get("link").cloned().unwrap_or(json!(url)),
                    "region": region,
                }));
            }
        }
    }
    Ok(out)
}

fn extract_ovens_json(body: &str) -> Option<Vec<Value>> {
    let marker = "var ovens = ";
    let start = body.find(marker)? + marker.len();
    let rest = &body[start..];
    let end = rest.find(";</script>").or_else(|| rest.find(';'))?;
    serde_json::from_str::<Vec<Value>>(&rest[..end]).ok()
}
