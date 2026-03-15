use crate::store::DataStore;
use serde_json::{json, Value};
use std::{fs, sync::Arc};

pub async fn fetch(store: &Arc<DataStore>) -> anyhow::Result<()> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(20))
        .build()?;

    if let Ok(outages) = fetch_outages(&client).await {
        store.set("internet_outages", json!(outages));
    }
    if let Ok(sdrs) = fetch_kiwisdr(&client).await {
        store.set("kiwisdr", json!(sdrs));
    }
    if let Ok(cctv) = fetch_cctv(&client).await {
        store.set("cctv", json!(cctv));
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

async fn fetch_kiwisdr(client: &reqwest::Client) -> anyhow::Result<Vec<Value>> {
    let resp = client
        .get("http://rx.linkfanel.net/kiwisdr_com.js")
        .send()
        .await?;
    if !resp.status().is_success() {
        return Ok(Vec::new());
    }

    let body = resp.text().await?;
    let Some(start) = body.find('[') else { return Ok(Vec::new()) };
    let Some(end) = body.rfind(']') else { return Ok(Vec::new()) };
    let arr = serde_json::from_str::<Vec<Value>>(&body[start..=end]).unwrap_or_default();

    Ok(arr
        .into_iter()
        .filter_map(|item| {
            let gps = item.get("gps")?.as_str()?;
            let parts: Vec<_> = gps.split(',').collect();
            let lat = parts.first()?.trim().parse::<f64>().ok()?;
            let lng = parts.get(1)?.trim().parse::<f64>().ok()?;
            Some(json!({
                "name": item.get("name"),
                "lat": lat,
                "lng": lng,
                "url": item.get("url"),
                "antenna": item.get("antenna"),
                "bands": item.get("bands"),
                "users": item.get("users"),
                "users_max": item.get("users_max"),
            }))
        })
        .collect())
}

async fn fetch_cctv(client: &reqwest::Client) -> anyhow::Result<Vec<Value>> {
    let mut cameras = Vec::new();

    if let Ok(resp) = client.get("https://api.data.gov.sg/v1/transport/traffic-images").send().await {
        if resp.status().is_success() {
            if let Ok(data) = resp.json::<Value>().await {
                if let Some(items) = data.get("items").and_then(Value::as_array).and_then(|v| v.first()) {
                    if let Some(arr) = items.get("cameras").and_then(Value::as_array) {
                        for cam in arr.iter().take(80) {
                            cameras.push(json!({
                                "id": format!("sg-{}", cam.get("camera_id").and_then(Value::as_str).unwrap_or("")),
                                "lat": cam.get("location").and_then(|v| v.get("latitude")).cloned().unwrap_or(Value::Null),
                                "lng": cam.get("location").and_then(|v| v.get("longitude")).cloned().unwrap_or(Value::Null),
                                "source": "Singapore LTA",
                                "media_url": cam.get("image").cloned().unwrap_or(Value::Null),
                                "direction": cam.get("camera_id").cloned().unwrap_or(Value::Null),
                            }));
                        }
                    }
                }
            }
        }
    }

    if let Ok(resp) = client.get("https://api.tfl.gov.uk/Place/Type/JamCam").send().await {
        if resp.status().is_success() {
            if let Ok(data) = resp.json::<Vec<Value>>().await {
                for cam in data.into_iter().take(120) {
                    let media = cam
                        .get("additionalProperties")
                        .and_then(Value::as_array)
                        .and_then(|props| {
                            props.iter().find_map(|prop| {
                                let key = prop.get("key").and_then(Value::as_str)?;
                                if key == "imageUrl" || key == "videoUrl" {
                                    prop.get("value").cloned()
                                } else {
                                    None
                                }
                            })
                        });
                    cameras.push(json!({
                        "id": format!("tfl-{}", cam.get("id").and_then(Value::as_str).unwrap_or("")),
                        "lat": cam.get("lat").cloned().unwrap_or(Value::Null),
                        "lng": cam.get("lon").cloned().unwrap_or(Value::Null),
                        "source": "TfL JamCam",
                        "media_url": media.unwrap_or(Value::Null),
                        "direction": cam.get("commonName").cloned().unwrap_or(Value::Null),
                    }));
                }
            }
        }
    }

    Ok(cameras)
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
