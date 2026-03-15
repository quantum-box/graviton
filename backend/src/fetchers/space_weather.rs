use crate::store::DataStore;
use serde_json::{json, Value};
use std::sync::Arc;

pub async fn fetch(store: &Arc<DataStore>) -> anyhow::Result<()> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()?;

    // Fetch Kp index from NOAA SWPC
    let resp = client
        .get("https://services.swpc.noaa.gov/products/noaa-planetary-k-index.json")
        .send()
        .await?;

    let mut kp_index = 0.0f64;

    if resp.status().is_success() {
        if let Ok(data) = resp.json::<Vec<Vec<Value>>>().await {
            // Last entry (skip header row)
            if let Some(last) = data.last() {
                kp_index = last
                    .get(1)
                    .and_then(|v| v.as_str())
                    .and_then(|s| s.parse::<f64>().ok())
                    .unwrap_or(0.0);
            }
        }
    }

    let status = match kp_index as u32 {
        0..=3 => "quiet",
        4 => "active",
        5 => "minor_storm",
        6 => "moderate_storm",
        7 => "strong_storm",
        _ => "severe_storm",
    };

    store.set(
        "space_weather",
        json!({
            "kp_index": kp_index,
            "status": status,
            "solar_events": [],
        }),
    );
    store.update_etag("slow");
    tracing::info!("Space weather: Kp={}, status={}", kp_index, status);
    Ok(())
}
