use anyhow::Result;
use regex::Regex;
use serde_json::{json, Value};

pub async fn fetch_broadcastify_top(client: &reqwest::Client) -> Result<Vec<Value>> {
    let resp = client
        .get("https://www.broadcastify.com/listen/top")
        .header("User-Agent", "Mozilla/5.0")
        .send()
        .await?;
    if !resp.status().is_success() {
        return Ok(Vec::new());
    }
    let html = resp.text().await?;
    let row_re = Regex::new(r#"/listen/feed/(\d+)[^>]*>([^<]+)</a>"#)?;
    let mut feeds = Vec::new();
    for cap in row_re.captures_iter(&html).take(50) {
        let id = cap.get(1).map(|m| m.as_str()).unwrap_or_default();
        let name = cap.get(2).map(|m| m.as_str()).unwrap_or("Broadcastify Feed");
        feeds.push(json!({
            "id": id,
            "name": name.trim(),
            "category": "Broadcastify",
            "stream_url": format!("https://broadcastify.cdnstream1.com/{}", id),
        }));
    }
    Ok(feeds)
}

pub async fn openmhz_systems(client: &reqwest::Client) -> Result<Value> {
    let resp = client.get("https://api.openmhz.com/systems").send().await?;
    if !resp.status().is_success() {
        return Ok(json!([]));
    }
    Ok(resp.json::<Value>().await.unwrap_or_else(|_| json!([])))
}

pub async fn openmhz_calls(client: &reqwest::Client, sys_name: &str) -> Result<Value> {
    let resp = client.get(format!("https://api.openmhz.com/{}/calls", sys_name)).send().await?;
    if !resp.status().is_success() {
        return Ok(json!([]));
    }
    Ok(resp.json::<Value>().await.unwrap_or_else(|_| json!([])))
}

pub async fn nearest_systems(client: &reqwest::Client, lat: f64, lng: f64, limit: usize) -> Result<Vec<Value>> {
    let systems = openmhz_systems(client).await?;
    let arr = systems
        .get("systems")
        .and_then(Value::as_array)
        .cloned()
        .or_else(|| systems.as_array().cloned())
        .unwrap_or_default();
    let mut scored: Vec<(f64, Value)> = arr
        .into_iter()
        .filter_map(|item| {
            let s_lat = item.get("lat").and_then(Value::as_f64)?;
            let s_lng = item.get("lng").and_then(Value::as_f64)?;
            Some((haversine_miles(lat, lng, s_lat, s_lng), item))
        })
        .collect();
    scored.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    Ok(scored
        .into_iter()
        .take(limit)
        .map(|(distance_miles, mut item)| {
            item["distance_miles"] = json!((distance_miles * 10.0).round() / 10.0);
            item
        })
        .collect())
}

fn haversine_miles(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let r = 3958.8_f64;
    let d_lat = (lat2 - lat1).to_radians();
    let d_lon = (lon2 - lon1).to_radians();
    let a = (d_lat / 2.0).sin().powi(2)
        + lat1.to_radians().cos() * lat2.to_radians().cos() * (d_lon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
    r * c
}

