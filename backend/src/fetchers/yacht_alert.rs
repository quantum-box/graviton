use serde_json::{json, Value};
use std::{collections::HashMap, fs, sync::OnceLock};

static YACHT_ALERT_DB: OnceLock<HashMap<String, Value>> = OnceLock::new();

pub fn yacht_alert_db() -> &'static HashMap<String, Value> {
    YACHT_ALERT_DB.get_or_init(|| {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/data/yacht_alert_db.json");
        fs::read_to_string(path)
            .ok()
            .and_then(|raw| serde_json::from_str(&raw).ok())
            .unwrap_or_default()
    })
}

pub fn enrich_ship(mut ship: Value) -> Value {
    let mmsi = ship.get("mmsi").and_then(Value::as_str).unwrap_or("").trim();
    if let Some(info) = yacht_alert_db().get(mmsi) {
        ship["yacht_alert"] = json!(true);
        ship["yacht_owner"] = info.get("owner").cloned().unwrap_or(Value::Null);
        ship["yacht_name"] = info.get("name").cloned().unwrap_or(Value::Null);
        ship["yacht_category"] = info.get("category").cloned().unwrap_or(Value::Null);
        ship["yacht_color"] = json!(match info.get("category").and_then(Value::as_str).unwrap_or("") {
            "Oligarch Watch" => "#FF2020",
            _ => "#FF69B4",
        });
        ship["yacht_builder"] = info.get("builder").cloned().unwrap_or(Value::Null);
        ship["yacht_length"] = info.get("length_m").cloned().unwrap_or(Value::Null);
        ship["yacht_year"] = info.get("year").cloned().unwrap_or(Value::Null);
        ship["yacht_link"] = info.get("link").cloned().unwrap_or(Value::Null);
    }
    ship
}

