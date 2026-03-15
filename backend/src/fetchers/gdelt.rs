use crate::store::DataStore;
use serde_json::{json, Value};
use std::sync::Arc;

pub async fn fetch(store: &Arc<DataStore>) -> anyhow::Result<()> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    // Fetch GDELT events from GKG (Global Knowledge Graph) API
    let resp = client
        .get("https://api.gdeltproject.org/api/v2/geo/geo?query=conflict%20OR%20attack%20OR%20military&mode=pointdata&format=geojson&timespan=1d&maxpoints=500")
        .send()
        .await?;

    if !resp.status().is_success() {
        anyhow::bail!("GDELT returned {}", resp.status());
    }

    let data: Value = resp.json().await?;
    let mut events: Vec<Value> = Vec::new();

    if let Some(features) = data.get("features").and_then(|v| v.as_array()) {
        for f in features {
            let props = f.get("properties").unwrap_or(&Value::Null);
            let geom = f.get("geometry").unwrap_or(&Value::Null);
            let coords = geom.get("coordinates").and_then(|c| c.as_array());

            if let Some(coords) = coords {
                let lng = coords.first().and_then(|v| v.as_f64()).unwrap_or(0.0);
                let lat = coords.get(1).and_then(|v| v.as_f64()).unwrap_or(0.0);

                events.push(json!({
                    "lat": lat,
                    "lng": lng,
                    "description": props.get("name").or(props.get("html")),
                    "source_url": props.get("url"),
                    "num_mentions": props.get("count").and_then(|v| v.as_u64()).unwrap_or(1),
                    "date": props.get("date"),
                }));
            }
        }
    }

    store.set("gdelt", json!(events));
    store.update_etag("slow");
    tracing::info!("GDELT updated: {} events", events.len());
    Ok(())
}
