use crate::store::DataStore;
use serde_json::{json, Value};
use std::{fs, sync::Arc};

const SAT_KEYWORDS: &[(&str, &str, &str)] = &[
    ("USA", "military_recon", "USA"),
    ("NROL", "sigint", "USA"),
    ("SBIRS", "early_warning", "USA"),
    ("YAOGAN", "military_recon", "China"),
    ("GAOFEN", "commercial_imaging", "China"),
    ("ICEYE", "sar", "Finland"),
    ("CAPELLA", "sar", "USA"),
    ("WORLDVIEW", "commercial_imaging", "USA"),
    ("SKYSAT", "commercial_imaging", "USA"),
    ("SENTINEL", "earth_observation", "EU"),
    ("NAVSTAR", "navigation", "USA"),
    ("GLONASS", "navigation", "Russia"),
    ("BEIDOU", "navigation", "China"),
    ("GALILEO", "navigation", "EU"),
    ("ISS", "space_station", "Intl"),
    ("TIANGONG", "space_station", "China"),
];

pub async fn fetch(store: &Arc<DataStore>) -> anyhow::Result<()> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    let data = fetch_satellite_cache(&client)
        .await
        .or_else(|_| load_disk_cache())
        .unwrap_or_default();

    let satellites: Vec<Value> = data
        .into_iter()
        .filter_map(|item| classify_satellite(item))
        .take(1200)
        .collect();

    store.set("satellites", json!(satellites));
    store.set("satellite_source", json!("celestrak_cache"));
    store.update_etag("fast");
    Ok(())
}

async fn fetch_satellite_cache(client: &reqwest::Client) -> anyhow::Result<Vec<Value>> {
    let resp = client
        .get("https://celestrak.org/NORAD/elements/gp.php?GROUP=active&FORMAT=json")
        .header("User-Agent", "Graviton/1.0")
        .send()
        .await?;
    if !resp.status().is_success() {
        anyhow::bail!("CelesTrak returned {}", resp.status());
    }
    let data = resp.json::<Vec<Value>>().await?;
    fs::write(
        concat!(env!("CARGO_MANIFEST_DIR"), "/data/sat_gp_cache.json"),
        serde_json::to_vec(&data)?,
    )?;
    Ok(data)
}

fn load_disk_cache() -> anyhow::Result<Vec<Value>> {
    let raw = fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/data/sat_gp_cache.json"))?;
    Ok(serde_json::from_str(&raw)?)
}

fn classify_satellite(item: Value) -> Option<Value> {
    let name = item.get("OBJECT_NAME")?.as_str()?.to_string();
    let mut mission = "general";
    let mut country = "Unknown";
    for (needle, m, c) in SAT_KEYWORDS {
        if name.to_uppercase().contains(needle) {
            mission = m;
            country = c;
            break;
        }
    }

    let mm = item.get("MEAN_MOTION").and_then(Value::as_f64).unwrap_or(0.0);
    let incl = item.get("INCLINATION").and_then(Value::as_f64).unwrap_or(0.0);
    let raan = item.get("RA_OF_ASC_NODE").and_then(Value::as_f64).unwrap_or(0.0);
    let ma = item.get("MEAN_ANOMALY").and_then(Value::as_f64).unwrap_or(0.0);

    // Lightweight pseudo-projection from orbital elements.
    let phase = (chrono::Utc::now().timestamp() as f64 / 60.0 + ma) % 360.0;
    let lat = (phase.to_radians().sin() * incl.sin().abs() * 90.0).clamp(-85.0, 85.0);
    let lng = ((raan + phase + 540.0) % 360.0) - 180.0;
    let altitude_km = mean_motion_to_altitude_km(mm);
    let speed_km_s = if altitude_km > 30000.0 { 3.1 } else { 7.4 };

    Some(json!({
        "id": item.get("NORAD_CAT_ID").and_then(Value::as_u64).unwrap_or(0),
        "norad_id": item.get("NORAD_CAT_ID").and_then(Value::as_u64).unwrap_or(0),
        "name": name,
        "mission": mission,
        "sat_type": mission,
        "country": country,
        "lat": (lat * 100000.0).round() / 100000.0,
        "lng": (lng * 100000.0).round() / 100000.0,
        "alt_km": altitude_km,
        "altitude_km": altitude_km,
        "speed_knots": speed_km_s * 1943.84,
        "speed_km_s": speed_km_s,
        "heading": phase,
    }))
}

fn mean_motion_to_altitude_km(mean_motion: f64) -> f64 {
    if mean_motion <= 0.0 {
        return 500.0;
    }
    let period_seconds = 86400.0 / mean_motion;
    let mu = 398600.4418_f64;
    let semi_major = (mu * (period_seconds / (2.0 * std::f64::consts::PI)).powi(2)).cbrt();
    ((semi_major - 6378.137) * 10.0).round() / 10.0
}
