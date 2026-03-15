use crate::store::DataStore;
use serde_json::{json, Value};
use std::sync::Arc;

pub async fn fetch(store: &Arc<DataStore>) -> anyhow::Result<()> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()?;

    let resp = client
        .get("https://earthquake.usgs.gov/earthquakes/feed/v1.0/summary/2.5_day.geojson")
        .send()
        .await?;

    let data: Value = resp.json().await?;
    let mut quakes: Vec<Value> = Vec::new();

    if let Some(features) = data.get("features").and_then(|v| v.as_array()) {
        for f in features {
            let props = f.get("properties").unwrap_or(&Value::Null);
            let geom = f.get("geometry").unwrap_or(&Value::Null);
            let coords = geom.get("coordinates").and_then(|c| c.as_array());

            if let Some(coords) = coords {
                let lng = coords.first().and_then(|v| v.as_f64()).unwrap_or(0.0);
                let lat = coords.get(1).and_then(|v| v.as_f64()).unwrap_or(0.0);
                let depth = coords.get(2).and_then(|v| v.as_f64()).unwrap_or(0.0);

                quakes.push(json!({
                    "id": f.get("id").and_then(|v| v.as_str()).unwrap_or(""),
                    "lat": lat,
                    "lng": lng,
                    "magnitude": props.get("mag").and_then(|v| v.as_f64()).unwrap_or(0.0),
                    "depth_km": depth,
                    "place": props.get("place").and_then(|v| v.as_str()).unwrap_or("Unknown"),
                    "time": props.get("time").and_then(|v| v.as_i64())
                        .map(|t| chrono::DateTime::from_timestamp_millis(t)
                            .map(|d| d.to_rfc3339())
                            .unwrap_or_default())
                        .unwrap_or_default(),
                    "url": props.get("url").and_then(|v| v.as_str()),
                }));
            }
        }
    }

    store.set("earthquakes", json!(quakes));
    store.update_etag("slow");
    tracing::info!("Earthquakes updated: {} events", quakes.len());
    Ok(())
}
