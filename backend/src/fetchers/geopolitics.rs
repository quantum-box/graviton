use anyhow::Result;
use serde_json::{json, Value};

pub async fn fetch_frontlines(client: &reqwest::Client) -> Result<Value> {
    let tree_resp = client
        .get("https://api.github.com/repos/cyterat/deepstate-map-data/git/trees/main?recursive=1")
        .header("User-Agent", "Graviton/1.0")
        .send()
        .await?;
    if !tree_resp.status().is_success() {
        return Ok(json!({"type":"FeatureCollection","features":[]}));
    }
    let tree: Value = tree_resp.json().await?;
    let path = tree
        .get("tree")
        .and_then(Value::as_array)
        .and_then(|items| {
            items.iter().find_map(|item| {
                let p = item.get("path").and_then(Value::as_str)?;
                if p.ends_with(".geojson") && (p.contains("line") || p.contains("front")) {
                    Some(p.to_string())
                } else {
                    None
                }
            })
        });
    let Some(path) = path else {
        return Ok(json!({"type":"FeatureCollection","features":[]}));
    };
    let raw_url = format!("https://raw.githubusercontent.com/cyterat/deepstate-map-data/main/{}", path);
    let resp = client.get(raw_url).send().await?;
    if !resp.status().is_success() {
        return Ok(json!({"type":"FeatureCollection","features":[]}));
    }
    Ok(resp.json::<Value>().await.unwrap_or_else(|_| json!({"type":"FeatureCollection","features":[]})))
}

