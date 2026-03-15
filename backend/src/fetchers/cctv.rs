use anyhow::Result;
use serde_json::{json, Value};

pub async fn fetch(client: &reqwest::Client) -> Result<Vec<Value>> {
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

