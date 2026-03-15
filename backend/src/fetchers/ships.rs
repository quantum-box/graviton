use crate::store::DataStore;
use serde_json::{json, Value};
use std::{collections::HashMap, fs, sync::Arc};

const CARRIER_ESTIMATES: &[(&str, &str, f64, f64, &str, &str)] = &[
    ("338000001", "USS Nimitz (CVN-68)", 47.5535, -122.6400, "carrier", "Bremerton, WA"),
    ("338000002", "USS Dwight D. Eisenhower (CVN-69)", 36.9465, -76.3265, "carrier", "Norfolk, VA"),
    ("338000003", "USS Carl Vinson (CVN-70)", 32.6840, -117.1290, "carrier", "San Diego, CA"),
    ("338000004", "USS Theodore Roosevelt (CVN-71)", 32.6885, -117.1280, "carrier", "San Diego, CA"),
    ("338000005", "USS Abraham Lincoln (CVN-72)", 20.0000, 64.0000, "carrier", "Arabian Sea"),
    ("338000006", "USS George Washington (CVN-73)", 35.2830, 139.6700, "carrier", "Yokosuka"),
    ("338000007", "USS John C. Stennis (CVN-74)", 36.9800, -76.4300, "carrier", "Newport News"),
    ("338000008", "USS Harry S. Truman (CVN-75)", 36.0000, 15.0000, "carrier", "Mediterranean"),
    ("338000009", "USS Ronald Reagan (CVN-76)", 47.5580, -122.6360, "carrier", "Bremerton, WA"),
    ("338000010", "USS George H.W. Bush (CVN-77)", 36.5000, -74.0000, "carrier", "Atlantic"),
    ("338000011", "USS Gerald R. Ford (CVN-78)", 18.0000, 39.5000, "carrier", "Red Sea"),
];

pub async fn fetch(store: &Arc<DataStore>) -> anyhow::Result<()> {
    let mut ships = carrier_positions();
    ships.extend(store.get("ais_feed_snapshot").and_then(|v| v.as_array().cloned()).unwrap_or_default());

    let yacht_db = yacht_alert_db();
    for ship in &mut ships {
        if let Some(mmsi) = ship.get("mmsi").and_then(Value::as_str) {
            if let Some(info) = yacht_db.get(mmsi) {
                ship["yacht_alert"] = json!(true);
                ship["yacht_owner"] = info.get("owner").cloned().unwrap_or(Value::Null);
                ship["yacht_name"] = info.get("name").cloned().unwrap_or(Value::Null);
                ship["yacht_category"] = info.get("category").cloned().unwrap_or(Value::Null);
                ship["yacht_link"] = info.get("link").cloned().unwrap_or(Value::Null);
                ship["type"] = json!("yacht");
            }
        }
    }

    store.set("ships", json!(ships));
    store.update_etag("fast");
    Ok(())
}

fn carrier_positions() -> Vec<Value> {
    CARRIER_ESTIMATES
        .iter()
        .map(|(mmsi, name, lat, lng, ship_type, desc)| {
            json!({
                "mmsi": mmsi,
                "name": name,
                "type": ship_type,
                "lat": lat,
                "lng": lng,
                "heading": 0,
                "sog": 0,
                "cog": 0,
                "country": "United States",
                "estimated": true,
                "source": "carrier_tracker",
                "desc": desc,
            })
        })
        .collect()
}

fn yacht_alert_db() -> HashMap<String, Value> {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/data/yacht_alert_db.json");
    fs::read_to_string(path)
        .ok()
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .unwrap_or_default()
}
