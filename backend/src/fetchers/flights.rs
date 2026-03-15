use crate::store::DataStore;
use serde_json::{json, Value};
use std::sync::Arc;

pub async fn fetch(store: &Arc<DataStore>) -> anyhow::Result<()> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()?;

    // Primary source: adsb.lol military flights
    let mut military: Vec<Value> = Vec::new();
    let mut commercial: Vec<Value> = Vec::new();

    // Fetch from adsb.lol (mil)
    match client.get("https://api.adsb.lol/v2/mil").send().await {
        Ok(resp) if resp.status().is_success() => {
            if let Ok(data) = resp.json::<Value>().await {
                if let Some(ac) = data.get("ac").and_then(|v| v.as_array()) {
                    for a in ac {
                        if let (Some(lat), Some(lon)) = (
                            a.get("lat").and_then(|v| v.as_f64()),
                            a.get("lon").and_then(|v| v.as_f64()),
                        ) {
                            military.push(json!({
                                "icao": a.get("hex").and_then(|v| v.as_str()).unwrap_or(""),
                                "callsign": a.get("flight").and_then(|v| v.as_str()).map(|s| s.trim()),
                                "lat": (lat * 100000.0).round() / 100000.0,
                                "lng": (lon * 100000.0).round() / 100000.0,
                                "altitude": a.get("alt_baro").and_then(|v| v.as_f64()),
                                "heading": a.get("track").and_then(|v| v.as_f64()),
                                "speed": a.get("gs").and_then(|v| v.as_f64()),
                                "category": "military",
                                "aircraft_type": a.get("t").and_then(|v| v.as_str()),
                                "registration": a.get("r").and_then(|v| v.as_str()),
                                "operator": a.get("ownOp").and_then(|v| v.as_str()),
                                "on_ground": a.get("alt_baro").and_then(|v| v.as_str()) == Some("ground"),
                                "squawk": a.get("squawk").and_then(|v| v.as_str()),
                            }));
                        }
                    }
                }
            }
        }
        _ => tracing::warn!("Failed to fetch adsb.lol mil"),
    }

    // Fetch from adsb.lol (LADD for interesting aircraft)
    match client.get("https://api.adsb.lol/v2/ladd").send().await {
        Ok(resp) if resp.status().is_success() => {
            if let Ok(data) = resp.json::<Value>().await {
                if let Some(ac) = data.get("ac").and_then(|v| v.as_array()) {
                    for a in ac {
                        if let (Some(lat), Some(lon)) = (
                            a.get("lat").and_then(|v| v.as_f64()),
                            a.get("lon").and_then(|v| v.as_f64()),
                        ) {
                            let cat = classify_aircraft(a);
                            let item = json!({
                                "icao": a.get("hex").and_then(|v| v.as_str()).unwrap_or(""),
                                "callsign": a.get("flight").and_then(|v| v.as_str()).map(|s| s.trim()),
                                "lat": (lat * 100000.0).round() / 100000.0,
                                "lng": (lon * 100000.0).round() / 100000.0,
                                "altitude": a.get("alt_baro").and_then(|v| v.as_f64()),
                                "heading": a.get("track").and_then(|v| v.as_f64()),
                                "speed": a.get("gs").and_then(|v| v.as_f64()),
                                "category": cat,
                                "aircraft_type": a.get("t").and_then(|v| v.as_str()),
                                "registration": a.get("r").and_then(|v| v.as_str()),
                                "operator": a.get("ownOp").and_then(|v| v.as_str()),
                                "on_ground": a.get("alt_baro").and_then(|v| v.as_str()) == Some("ground"),
                                "squawk": a.get("squawk").and_then(|v| v.as_str()),
                            });
                            commercial.push(item);
                        }
                    }
                }
            }
        }
        _ => {}
    }

    store.set("military_flights", json!(military));
    store.set("commercial_flights", json!(commercial));
    store.update_etag("fast");

    tracing::info!("Flights updated: {} military, {} commercial", military.len(), commercial.len());
    Ok(())
}

fn classify_aircraft(a: &Value) -> &'static str {
    let t = a.get("t").and_then(|v| v.as_str()).unwrap_or("");
    let callsign = a.get("flight").and_then(|v| v.as_str()).unwrap_or("");

    if t.contains("H60") || t.contains("H47") || t.contains("AH") || t.contains("UH")
        || t.contains("EC") || t.contains("R44") || t.contains("R22") || t.contains("B06")
        || t.contains("B47") || t.contains("A109") || t.contains("S76") || t.contains("AS35")
        || t.contains("H145") || t.contains("H135") || t.contains("EC35") || t.contains("EC45")
    {
        return "helicopter";
    }
    if t.starts_with("C1") || t.starts_with("C2") || t.starts_with("C5") || t.starts_with("F1")
        || t.starts_with("F2") || t.starts_with("F3") || t.starts_with("B1")
        || t.starts_with("B2") || t.starts_with("KC") || t.starts_with("E3")
        || t.starts_with("E6") || t.starts_with("E8") || t.starts_with("P8")
        || t.starts_with("RC") || t.contains("HAWK") || t.starts_with("MQ")
        || t.starts_with("RQ") || t.contains("GLBX")
    {
        return "military";
    }
    if t.contains("GLF") || t.contains("CL60") || t.contains("LJ") || t.contains("C680")
        || t.contains("C560") || t.contains("FA") || (t.contains('G') && t.len() <= 4)
        || t.contains("GLEX") || t.contains("BD70")
    {
        return "bizjet";
    }
    if callsign.len() >= 3 {
        let prefix = &callsign[..3];
        if ["AAL", "DAL", "UAL", "SWA", "BAW", "AFR", "DLH", "RYR", "EZY", "THY", "QFA",
            "SIA", "ANA", "JAL", "KAL", "CCA", "CSN", "CES"]
            .contains(&prefix)
        {
            return "airline";
        }
    }
    "general_aviation"
}
