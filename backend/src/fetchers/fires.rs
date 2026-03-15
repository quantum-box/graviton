use crate::store::DataStore;
use serde_json::{json, Value};
use std::sync::Arc;

pub async fn fetch(store: &Arc<DataStore>) -> anyhow::Result<()> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    let resp = client
        .get("https://firms.modaps.eosdis.nasa.gov/data/active_fire/noaa-20-viirs-c2/csv/J1_VIIRS_C2_Global_24h.csv")
        .send()
        .await?;

    if !resp.status().is_success() {
        anyhow::bail!("FIRMS returned {}", resp.status());
    }

    let body = resp.text().await?;
    let mut fires: Vec<Value> = Vec::new();
    let mut rdr = csv::Reader::from_reader(body.as_bytes());

    for result in rdr.records() {
        if let Ok(record) = result {
            let lat: f64 = record.get(0).and_then(|v| v.parse().ok()).unwrap_or(0.0);
            let lng: f64 = record.get(1).and_then(|v| v.parse().ok()).unwrap_or(0.0);
            let brightness: f64 = record.get(2).and_then(|v| v.parse().ok()).unwrap_or(0.0);
            let confidence = record.get(8).unwrap_or("low").to_string();
            let acq_date = record.get(5).unwrap_or("").to_string();
            let acq_time = record.get(6).unwrap_or("").to_string();
            let daynight = record.get(11).unwrap_or("").to_string();
            let frp: f64 = record.get(13).and_then(|v| v.parse().ok()).unwrap_or(0.0);

            if lat != 0.0 && lng != 0.0 {
                fires.push(json!({
                    "lat": lat,
                    "lng": lng,
                    "frp": frp,
                    "brightness": brightness,
                    "confidence": confidence,
                    "satellite": "NOAA-20",
                    "daynight": daynight,
                    "acq_date": acq_date,
                    "acq_time": acq_time,
                }));
            }
        }
    }

    // Sort by FRP descending, keep top 5000
    fires.sort_by(|a, b| {
        let fa = a.get("frp").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let fb = b.get("frp").and_then(|v| v.as_f64()).unwrap_or(0.0);
        fb.partial_cmp(&fa).unwrap_or(std::cmp::Ordering::Equal)
    });
    fires.truncate(5000);

    store.set("firms_fires", json!(fires));
    store.update_etag("slow");
    tracing::info!("Fires updated: {} hotspots", fires.len());
    Ok(())
}
