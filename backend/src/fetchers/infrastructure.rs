use crate::store::DataStore;
use serde_json::{json, Value};
use std::sync::Arc;

pub async fn fetch(store: &Arc<DataStore>) -> anyhow::Result<()> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()?;

    // Internet outages from IODA
    match fetch_outages(&client).await {
        Ok(outages) => {
            store.set("internet_outages", json!(outages));
            tracing::info!("Internet outages: {} regions", outages.len());
        }
        Err(e) => tracing::warn!("IODA fetch failed: {}", e),
    }

    // KiwiSDR receivers
    match fetch_kiwisdr(&client).await {
        Ok(sdrs) => {
            store.set("kiwisdr", json!(sdrs));
            tracing::info!("KiwiSDR: {} receivers", sdrs.len());
        }
        Err(e) => tracing::warn!("KiwiSDR fetch failed: {}", e),
    }

    store.update_etag("slow");
    Ok(())
}

async fn fetch_outages(client: &reqwest::Client) -> anyhow::Result<Vec<Value>> {
    let resp = client
        .get("https://api.ioda.inetintel.cc.gatech.edu/v2/signals/raw/region/AF,AS,EU,NA,SA,OC?sourceParams=bgp,ping-slash24&from=-24h&until=now")
        .send()
        .await?;

    let mut outages = Vec::new();

    if resp.status().is_success() {
        let data: Value = resp.json().await?;
        if let Some(results) = data.get("data").and_then(|d| d.as_array()) {
            for item in results {
                if let Some(entity) = item.get("entity") {
                    outages.push(json!({
                        "region": entity.get("name").and_then(|v| v.as_str()).unwrap_or(""),
                        "country": entity.get("attrs").and_then(|a| a.get("country_name")).and_then(|v| v.as_str()).unwrap_or(""),
                        "severity": item.get("severity").and_then(|v| v.as_f64()).unwrap_or(0.0),
                        "datasource": item.get("datasource").and_then(|v| v.as_str()).unwrap_or(""),
                    }));
                }
            }
        }
    }

    Ok(outages)
}

async fn fetch_kiwisdr(client: &reqwest::Client) -> anyhow::Result<Vec<Value>> {
    let resp = client
        .get("http://rx.linkfanel.net/kiwisdr_com.js")
        .send()
        .await?;

    let mut sdrs = Vec::new();

    if resp.status().is_success() {
        let body = resp.text().await?;
        // Parse the JavaScript-style data
        // The data is in var kiwisdr_com = [...]; format
        if let Some(start) = body.find('[') {
            if let Some(end) = body.rfind(']') {
                if let Ok(arr) = serde_json::from_str::<Vec<Value>>(&body[start..=end]) {
                    for item in arr {
                        if let (Some(lat), Some(lng)) = (
                            item.get("gps")
                                .and_then(|g| g.as_str())
                                .and_then(|s| {
                                    let parts: Vec<&str> = s.split(',').collect();
                                    parts
                                        .first()
                                        .and_then(|v| v.trim().parse::<f64>().ok())
                                }),
                            item.get("gps")
                                .and_then(|g| g.as_str())
                                .and_then(|s| {
                                    let parts: Vec<&str> = s.split(',').collect();
                                    parts
                                        .get(1)
                                        .and_then(|v| v.trim().parse::<f64>().ok())
                                }),
                        ) {
                            sdrs.push(json!({
                                "name": item.get("name"),
                                "lat": lat,
                                "lng": lng,
                                "url": item.get("url"),
                                "antenna": item.get("antenna"),
                                "bands": item.get("bands"),
                                "users": item.get("users"),
                                "users_max": item.get("users_max"),
                            }));
                        }
                    }
                }
            }
        }
    }

    Ok(sdrs)
}
