use anyhow::Result;
use serde_json::{json, Value};

pub async fn search(client: &reqwest::Client, lat: f64, lng: f64) -> Result<Value> {
    let body = json!({
        "collections": ["sentinel-2-l2a"],
        "intersects": {
            "type": "Point",
            "coordinates": [lng, lat],
        },
        "limit": 6,
        "sortby": [{"field": "datetime", "direction": "desc"}],
        "query": {
            "eo:cloud_cover": {"lt": 30}
        }
    });
    let resp = client
        .post("https://planetarycomputer.microsoft.com/api/stac/v1/search")
        .json(&body)
        .send()
        .await?;
    if !resp.status().is_success() {
        return Ok(json!({"features": []}));
    }
    Ok(resp.json::<Value>().await.unwrap_or_else(|_| json!({"features": []})))
}

