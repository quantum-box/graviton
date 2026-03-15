use crate::store::DataStore;
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, HashMap},
    sync::Arc,
};

use super::{military, plane_alert, retry};

pub async fn fetch(store: &Arc<DataStore>) -> anyhow::Result<()> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()?;

    let mut commercial = Vec::new();
    let mut private_flights = Vec::new();
    let mut private_jets = Vec::new();
    let mut military_flights = Vec::new();
    let mut tracked = Vec::new();
    let mut uavs = Vec::new();
    let mut jamming_samples = Vec::new();

    for (url, source) in [
        ("https://api.adsb.lol/v2/mil", "adsb_lol_mil"),
        ("https://api.adsb.lol/v2/ladd", "adsb_lol_ladd"),
    ] {
        match retry::with_retry_async(source, 2, || async { Ok(client.get(url).send().await?) }).await {
            Ok(resp) if resp.status().is_success() => {
                if let Ok(data) = resp.json::<Value>().await {
                    if let Some(ac) = data.get("ac").and_then(Value::as_array) {
                        for aircraft in ac {
                            if let Some(item) = normalize_aircraft(aircraft, source) {
                                if let Some(zone) = jamming_sample(aircraft, &item) {
                                    jamming_samples.push(zone);
                                }
                                if plane_alert::is_tracked(&item) {
                                    tracked.push(plane_alert::enrich_tracked(item.clone()));
                                }
                                match item.get("type").and_then(Value::as_str).unwrap_or("commercial_flight") {
                                    "military_flight" => military_flights.push(item),
                                    "uav" => uavs.push(item),
                                    "private_jet" => private_jets.push(item),
                                    "private_flight" => private_flights.push(item),
                                    _ => commercial.push(item),
                                }
                            }
                        }
                    }
                }
            }
            Ok(resp) => tracing::warn!("Flights source {} returned {}", source, resp.status()),
            Err(err) => tracing::warn!("Flights source {} failed: {}", source, err),
        }
    }

    let opensky = fetch_opensky(&client).await.unwrap_or_default();
    for aircraft in opensky {
        if let Some(item) = normalize_aircraft(&aircraft, "opensky") {
            if plane_alert::is_tracked(&item) {
                tracked.push(plane_alert::enrich_tracked(item.clone()));
            }
            commercial.push(item);
        }
    }

    dedupe_by_icao(&mut commercial);
    dedupe_by_icao(&mut private_flights);
    dedupe_by_icao(&mut private_jets);
    dedupe_by_icao(&mut military_flights);
    dedupe_by_icao(&mut tracked);
    dedupe_by_icao(&mut uavs);

    store.set("commercial_flights", json!(commercial));
    store.set("private_flights", json!(private_flights));
    store.set("private_jets", json!(private_jets));
    store.set("military_flights", json!(military_flights));
    store.set("tracked_flights", json!(tracked));
    store.set("uavs", json!(uavs));
    store.set("gps_jamming", json!(aggregate_jamming(jamming_samples)));
    store.update_etag("fast");
    Ok(())
}

fn normalize_aircraft(aircraft: &Value, source: &str) -> Option<Value> {
    let lat = aircraft.get("lat").and_then(Value::as_f64)?;
    let lng = aircraft.get("lon").and_then(Value::as_f64)?;
    let icao = aircraft
        .get("hex")
        .or_else(|| aircraft.get("icao24"))
        .and_then(Value::as_str)?
        .trim()
        .to_uppercase();
    if icao.is_empty() {
        return None;
    }
    let callsign = aircraft
        .get("flight")
        .or_else(|| aircraft.get("callsign"))
        .and_then(Value::as_str)
        .map(str::trim)
        .unwrap_or("");
    let registration = aircraft
        .get("r")
        .or_else(|| aircraft.get("registration"))
        .and_then(Value::as_str)
        .unwrap_or("");
    let aircraft_type = aircraft.get("t").and_then(Value::as_str).unwrap_or("");
    let operator = aircraft
        .get("ownOp")
        .or_else(|| aircraft.get("operator"))
        .and_then(Value::as_str)
        .unwrap_or("");

    let kind = military::classify_aircraft(aircraft_type, callsign, operator);
    let mut item = json!({
        "icao24": icao,
        "callsign": if callsign.is_empty() { Value::Null } else { json!(callsign) },
        "lat": round5(lat),
        "lng": round5(lng),
        "alt": aircraft.get("alt_baro").and_then(as_f64_loose),
        "heading": aircraft.get("track").and_then(as_f64_loose),
        "speed_knots": aircraft.get("gs").and_then(as_f64_loose),
        "registration": if registration.is_empty() { Value::Null } else { json!(registration.to_uppercase()) },
        "model": if aircraft_type.is_empty() { Value::Null } else { json!(aircraft_type) },
        "icao": aircraft_type,
        "operator": if operator.is_empty() { Value::Null } else { json!(operator) },
        "country": aircraft.get("country").and_then(Value::as_str),
        "squawk": aircraft.get("squawk").and_then(Value::as_str),
        "nac_p": aircraft.get("nac_p").and_then(as_u64_loose),
        "source": source,
        "type": kind,
    });
    if kind == "military_flight" {
        item["military_type"] = json!(military::classify_military_type(aircraft_type));
    }
    if kind == "uav" {
        let meta = military::uav_metadata(aircraft_type, callsign);
        item["uav_type"] = meta.get("uav_type").cloned().unwrap_or_else(|| json!("uav"));
        item["wiki"] = meta.get("wiki").cloned().unwrap_or(Value::Null);
    }
    Some(item)
}

fn jamming_sample(aircraft: &Value, item: &Value) -> Option<(i32, i32, bool)> {
    let nac_p = aircraft.get("nac_p").and_then(as_u64_loose)?;
    if nac_p > 4 {
        return None;
    }
    let lat = item.get("lat").and_then(Value::as_f64)?;
    let lng = item.get("lng").and_then(Value::as_f64)?;
    Some(((lat * 2.0).round() as i32, (lng * 2.0).round() as i32, nac_p <= 2))
}

fn aggregate_jamming(samples: Vec<(i32, i32, bool)>) -> Vec<Value> {
    let mut buckets: BTreeMap<(i32, i32), (u64, u64)> = BTreeMap::new();
    for (lat, lng, degraded) in samples {
        let entry = buckets.entry((lat, lng)).or_insert((0, 0));
        entry.0 += 1;
        if degraded {
            entry.1 += 1;
        }
    }
    buckets
        .into_iter()
        .filter(|(_, (total, _))| *total >= 3)
        .map(|((lat, lng), (total, degraded))| {
            let ratio = degraded as f64 / total as f64;
            json!({
                "lat": lat as f64 / 2.0,
                "lng": lng as f64 / 2.0,
                "severity": if ratio > 0.66 { "high" } else if ratio > 0.33 { "medium" } else { "low" },
                "ratio": (ratio * 100.0).round() / 100.0,
                "degraded": degraded,
                "total": total,
            })
        })
        .collect()
}

fn dedupe_by_icao(items: &mut Vec<Value>) {
    let mut seen = HashMap::new();
    items.retain(|item| {
        let key = item.get("icao24").and_then(Value::as_str).unwrap_or("").to_string();
        if key.is_empty() || seen.contains_key(&key) {
            false
        } else {
            seen.insert(key, ());
            true
        }
    });
}

async fn fetch_opensky(client: &reqwest::Client) -> anyhow::Result<Vec<Value>> {
    let mut req = client
        .get("https://opensky-network.org/api/states/all")
        .header("User-Agent", "Graviton/1.0");
    if let (Some(id), Some(secret)) = (
        std::env::var("OPENSKY_CLIENT_ID").ok(),
        std::env::var("OPENSKY_CLIENT_SECRET").ok(),
    ) {
        req = req.basic_auth(id, Some(secret));
    }
    let resp = req.send().await?;
    if !resp.status().is_success() {
        return Ok(Vec::new());
    }
    let data: Value = resp.json().await?;
    let mut out = Vec::new();
    if let Some(states) = data.get("states").and_then(Value::as_array) {
        for state in states.iter().take(250) {
            let Some(arr) = state.as_array() else { continue };
            let (Some(icao), Some(callsign), Some(lng), Some(lat)) = (
                arr.first().and_then(Value::as_str),
                arr.get(1).and_then(Value::as_str),
                arr.get(5).and_then(Value::as_f64),
                arr.get(6).and_then(Value::as_f64),
            ) else {
                continue;
            };
            out.push(json!({
                "hex": icao,
                "flight": callsign.trim(),
                "lon": lng,
                "lat": lat,
                "alt_baro": arr.get(7).and_then(Value::as_f64),
                "track": arr.get(10).and_then(Value::as_f64),
                "gs": arr.get(9).and_then(Value::as_f64).map(|v| v * 1.94384),
            }));
        }
    }
    Ok(out)
}

fn as_f64_loose(value: &Value) -> Option<f64> {
    value.as_f64().or_else(|| value.as_str()?.parse().ok())
}

fn as_u64_loose(value: &Value) -> Option<u64> {
    value.as_u64().or_else(|| value.as_str()?.parse().ok())
}

fn round5(v: f64) -> f64 {
    (v * 100000.0).round() / 100000.0
}
