use crate::store::DataStore;
use serde_json::{json, Value};
use std::sync::Arc;

const INTEL_SATS: &[(&str, u32, &str)] = &[
    ("ISS (ZARYA)", 25544, "space_station"),
    ("CSS (TIANHE)", 48274, "space_station"),
    ("STARLINK-30000", 56174, "commercial"),
    ("USA 224 (KH-11)", 37348, "military_recon"),
    ("USA 245 (KH-11)", 39232, "military_recon"),
    ("USA 290 (KH-11)", 44826, "military_recon"),
    ("USA 314 (KH-11)", 54473, "military_recon"),
    ("COSMOS 2558", 53328, "military_recon"),
    ("NROL-82", 48912, "military_recon"),
    ("USA 326", 58481, "military_recon"),
    ("GAOFEN 11-03", 50702, "military_recon"),
    ("YAOGAN 34-04A", 58487, "military_recon"),
    ("CSO-2", 49260, "military_recon"),
    ("KONDOR-FKA 1", 57756, "SAR"),
    ("COSMOS 2560", 54032, "SAR"),
    ("ICEYE-X14", 52761, "SAR"),
    ("CAPELLA-9", 53067, "SAR"),
    ("NAVSTAR 79 (GPS IIF-12)", 41019, "navigation"),
    ("BEIDOU-3 M23", 44793, "navigation"),
    ("GLONASS-K2 1", 55549, "navigation"),
    ("GALILEO 27", 48859, "navigation"),
    ("USA 200 (SBIRS GEO-1)", 37481, "early_warning"),
    ("COSMOS 2546", 45608, "early_warning"),
    ("LANDSAT 9", 49260, "earth_observation"),
    ("SENTINEL-2A", 40697, "earth_observation"),
    ("SENTINEL-2B", 42063, "earth_observation"),
    ("WORLDVIEW-3", 40115, "commercial_imaging"),
    ("PLANET SKYSAT-13", 43797, "commercial_imaging"),
];

pub async fn fetch(store: &Arc<DataStore>) -> anyhow::Result<()> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    let resp = client
        .get("https://celestrak.org/NORAD/elements/gp.php?GROUP=active&FORMAT=tle")
        .send()
        .await?;

    if !resp.status().is_success() {
        anyhow::bail!("CelesTrak returned {}", resp.status());
    }

    let body = resp.text().await?;
    let lines: Vec<&str> = body.lines().collect();

    let mut satellites: Vec<Value> = Vec::new();
    let now = chrono::Utc::now();

    let mut i = 0;
    while i + 2 < lines.len() {
        let name = lines[i].trim();
        let line1 = lines[i + 1].trim();
        let line2 = lines[i + 2].trim();
        i += 3;

        if !line1.starts_with('1') || !line2.starts_with('2') {
            i -= 2;
            continue;
        }

        // Extract NORAD ID from TLE line 1 (columns 3-7)
        let norad_id: u32 = line1
            .get(2..7)
            .and_then(|s| s.trim().parse().ok())
            .unwrap_or(0);

        let is_intel = INTEL_SATS.iter().any(|(_, id, _)| *id == norad_id);
        if !is_intel && norad_id % 50 != 0 && satellites.len() > 500 {
            continue;
        }

        if let Ok(elements) = sgp4::Elements::from_tle(
            Some(name.to_string()),
            line1.as_bytes(),
            line2.as_bytes(),
        ) {
            if let Ok(constants) = sgp4::Constants::from_elements(&elements) {
                let mins_since_epoch = minutes_since_epoch(&elements, &now);
                if let Ok(prediction) = constants.propagate(mins_since_epoch) {
                    let gmst = gmst_from_datetime(&now);
                    let (lat, lng, alt) = eci_to_geodetic(
                        prediction.position[0],
                        prediction.position[1],
                        prediction.position[2],
                        gmst,
                    );

                    let mission = INTEL_SATS
                        .iter()
                        .find(|(_, id, _)| *id == norad_id)
                        .map(|(_, _, m)| m.to_string());

                    let speed = (prediction.velocity[0].powi(2)
                        + prediction.velocity[1].powi(2)
                        + prediction.velocity[2].powi(2))
                    .sqrt();

                    satellites.push(json!({
                        "norad_id": norad_id,
                        "name": name,
                        "lat": (lat * 100000.0).round() / 100000.0,
                        "lng": (lng * 100000.0).round() / 100000.0,
                        "altitude_km": (alt * 10.0).round() / 10.0,
                        "speed_km_s": (speed * 100.0).round() / 100.0,
                        "mission": mission,
                    }));
                }
            }
        }
    }

    store.set("satellites", json!(satellites));
    store.update_etag("fast");
    tracing::info!("Satellites updated: {} tracked", satellites.len());
    Ok(())
}

fn minutes_since_epoch(elements: &sgp4::Elements, now: &chrono::DateTime<chrono::Utc>) -> f64 {
    // The sgp4 crate's Elements stores the epoch as a DateTime<Utc> via the epoch() method
    // We need to compute minutes from TLE epoch to now
    let epoch_year = elements.epoch_afspc_compatibility_mode().0 as i32;
    let epoch_year_full = if epoch_year >= 57 {
        1900 + epoch_year
    } else {
        2000 + epoch_year
    };

    let epoch_day = elements.epoch_afspc_compatibility_mode.1;

    // Build epoch datetime: Jan 1 of epoch_year + fractional days
    if let Some(jan1) = chrono::NaiveDate::from_ymd_opt(epoch_year_full, 1, 1) {
        let epoch_ndt =
            jan1.and_hms_opt(0, 0, 0).unwrap() + chrono::Duration::nanoseconds(((epoch_day - 1.0) * 86400.0 * 1e9) as i64);
        let epoch_utc =
            chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(epoch_ndt, chrono::Utc);
        let diff = *now - epoch_utc;
        diff.num_milliseconds() as f64 / 60000.0
    } else {
        0.0
    }
}

/// Compute Greenwich Mean Sidereal Time in radians from a UTC datetime
fn gmst_from_datetime(dt: &chrono::DateTime<chrono::Utc>) -> f64 {
    use chrono::Datelike;
    use chrono::Timelike;

    // Julian date calculation
    let y = dt.year() as f64;
    let m = dt.month() as f64;
    let d = dt.day() as f64;
    let h = dt.hour() as f64 + dt.minute() as f64 / 60.0 + dt.second() as f64 / 3600.0;

    let (y2, m2) = if m <= 2.0 {
        (y - 1.0, m + 12.0)
    } else {
        (y, m)
    };

    let jd = (365.25 * (y2 + 4716.0)).floor() + (30.6001 * (m2 + 1.0)).floor() + d + h / 24.0
        - 1524.5;

    // Julian centuries from J2000.0
    let t = (jd - 2451545.0) / 36525.0;

    // GMST in degrees
    let gmst_deg = 280.46061837 + 360.98564736629 * (jd - 2451545.0)
        + 0.000387933 * t * t
        - t * t * t / 38710000.0;

    (gmst_deg % 360.0).to_radians()
}

fn eci_to_geodetic(x: f64, y: f64, z: f64, gmst: f64) -> (f64, f64, f64) {
    let a = 6378.137; // Earth equatorial radius km
    let e2 = 0.00669437999014;

    let r = (x * x + y * y).sqrt();
    let mut lat = (z / r).atan();

    for _ in 0..10 {
        let sin_lat = lat.sin();
        let n = a / (1.0 - e2 * sin_lat * sin_lat).sqrt();
        lat = ((z + e2 * n * sin_lat) / r).atan();
    }

    let mut lng = y.atan2(x) - gmst;
    // Normalize longitude to [-pi, pi]
    lng = ((lng + std::f64::consts::PI) % (2.0 * std::f64::consts::PI)) - std::f64::consts::PI;
    if lng < -std::f64::consts::PI {
        lng += 2.0 * std::f64::consts::PI;
    }

    let sin_lat = lat.sin();
    let n = a / (1.0 - e2 * sin_lat * sin_lat).sqrt();
    let alt = if lat.cos().abs() > 1e-10 {
        r / lat.cos() - n
    } else {
        z.abs() - a * (1.0 - e2).sqrt()
    };

    (lat.to_degrees(), lng.to_degrees(), alt)
}
