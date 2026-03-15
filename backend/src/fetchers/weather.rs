use crate::store::DataStore;
use serde_json::{json, Value};
use std::sync::Arc;

pub async fn fetch(store: &Arc<DataStore>) -> anyhow::Result<()> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()?;

    // RainViewer API for radar timestamps
    let resp = client
        .get("https://api.rainviewer.com/public/weather-maps.json")
        .send()
        .await?;

    if resp.status().is_success() {
        if let Ok(data) = resp.json::<Value>().await {
            let radar = data
                .get("radar")
                .and_then(|r| r.get("past"))
                .and_then(|p| p.as_array())
                .and_then(|arr| arr.last())
                .cloned()
                .unwrap_or(Value::Null);

            store.set(
                "weather",
                json!({
                    "radar_tile_path": radar.get("path"),
                    "generated": data.get("generated"),
                    "host": data.get("host"),
                }),
            );
        }
    }

    store.update_etag("slow");
    Ok(())
}
