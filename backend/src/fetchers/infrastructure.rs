use crate::store::DataStore;
use serde_json::{json, Value};
use std::{fs, sync::Arc};

use super::{cctv, kiwisdr};

pub async fn fetch(store: &Arc<DataStore>) -> anyhow::Result<()> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(20))
        .build()?;

    if let Ok(outages) = fetch_outages(&client).await {
        store.set("internet_outages", json!(outages));
    }
    if let Ok(sdrs) = kiwisdr::fetch(&client).await {
        store.set("kiwisdr", json!(sdrs));
    }
    if let Ok(cameras) = cctv::fetch(&client).await {
        store.set("cctv", json!(cameras));
    }
    store.set("datacenters", json!(load_datacenters()));
    store.update_etag("slow");
    Ok(())
}

async fn fetch_outages(client: &reqwest::Client) -> anyhow::Result<Vec<Value>> {
    let resp = client
        .get("https://api.ioda.inetintel.cc.gatech.edu/v2/outages/recent")
        .send()
        .await?;
    if !resp.status().is_success() {
        return Ok(Vec::new());
    }
    let data: Value = resp.json().await?;
    let mut outages = Vec::new();
    if let Some(items) = data.get("outages").and_then(Value::as_array) {
        for item in items.iter().take(200) {
            outages.push(json!({
                "region_name": item.get("name").or_else(|| item.get("entity")).cloned().unwrap_or(Value::Null),
                "country_name": item.get("country_name").or_else(|| item.get("country")).cloned().unwrap_or(Value::Null),
                "severity": item.get("severity").and_then(Value::as_f64).unwrap_or(0.0),
                "datasource": item.get("datasource").or_else(|| item.get("source")).cloned().unwrap_or(Value::Null),
                "lat": item.get("latitude").or_else(|| item.get("lat")).cloned().unwrap_or(Value::Null),
                "lng": item.get("longitude").or_else(|| item.get("lng")).cloned().unwrap_or(Value::Null),
            }));
        }
    } else if let Some(items) = data.get("data").and_then(Value::as_array) {
        for item in items.iter().take(200) {
            let entity = item.get("entity").unwrap_or(&Value::Null);
            outages.push(json!({
                "region_name": entity.get("name").cloned().unwrap_or(Value::Null),
                "country_name": entity.get("attrs").and_then(|a| a.get("country_name")).cloned().unwrap_or(Value::Null),
                "severity": item.get("severity").and_then(Value::as_f64).unwrap_or(0.0),
                "datasource": item.get("datasource").cloned().unwrap_or(Value::Null),
                "lat": entity.get("attrs").and_then(|a| a.get("latitude")).cloned().unwrap_or(Value::Null),
                "lng": entity.get("attrs").and_then(|a| a.get("longitude")).cloned().unwrap_or(Value::Null),
            }));
        }
    }
    Ok(outages)
}

fn load_datacenters() -> Vec<Value> {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/data/datacenters_geocoded.json");
    let Ok(raw) = fs::read_to_string(path) else {
        return Vec::new();
    };
    let Ok(items) = serde_json::from_str::<Vec<Value>>(&raw) else {
        return Vec::new();
    };
    items
        .into_iter()
        .filter(|item| item.get("lat").and_then(Value::as_f64).is_some() && item.get("lng").and_then(Value::as_f64).is_some())
        .take(3000)
        .enumerate()
        .map(|(idx, item)| {
            json!({
                "id": format!("dc-{}", idx),
                "name": item.get("name").cloned().unwrap_or(Value::Null),
                "company": item.get("company").cloned().unwrap_or(Value::Null),
                "city": item.get("city").cloned().unwrap_or(Value::Null),
                "country": item.get("country").cloned().unwrap_or(Value::Null),
                "lat": item.get("lat").cloned().unwrap_or(Value::Null),
                "lng": item.get("lng").cloned().unwrap_or(Value::Null),
            })
        })
        .collect()
}
