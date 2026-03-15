use crate::store::DataStore;
use serde_json::{json, Value};
use std::sync::Arc;

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
                    "lat": coords.get(1).and_then(Value::as_f64).unwrap_or(0.0),
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
