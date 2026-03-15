use serde_json::{json, Value};
use std::{collections::HashMap, fs, sync::OnceLock};

static PLANE_ALERT_DB: OnceLock<HashMap<String, Value>> = OnceLock::new();
static TRACKED_NAMES_DB: OnceLock<HashMap<String, Value>> = OnceLock::new();

const POTUS_FLEET: &[(&str, &str, &str, &str, &str)] = &[
    ("ADFDF8", "#ff1493", "Air Force One (82-8000)", "Head of State", "AF1"),
    ("ADFDF9", "#ff1493", "Air Force One (92-9000)", "Head of State", "AF1"),
    ("ADFEB7", "blue", "Air Force Two (98-0001)", "Governments", "AF2"),
    ("AE0865", "#ff1493", "Marine One (VH-3D)", "Head of State", "M1"),
    ("AE5E76", "#ff1493", "Marine One (VH-92A)", "Head of State", "M1"),
];

pub fn plane_alert_db() -> &'static HashMap<String, Value> {
    PLANE_ALERT_DB.get_or_init(|| {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/data/plane_alert_db.json");
        let mut db: HashMap<String, Value> = fs::read_to_string(path)
            .ok()
            .and_then(|raw| serde_json::from_str(&raw).ok())
            .unwrap_or_default();

        for (icao, color, operator, category, fleet) in POTUS_FLEET {
            let entry = db.entry((*icao).to_string()).or_insert_with(|| json!({}));
            entry["color"] = json!(color);
            entry["operator"] = json!(operator);
            entry["category"] = json!(category);
            entry["potus_fleet"] = json!(fleet);
        }
        db
    })
}

pub fn tracked_names_db() -> &'static HashMap<String, Value> {
    TRACKED_NAMES_DB.get_or_init(|| {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/data/tracked_names.json");
        let Ok(raw) = fs::read_to_string(path) else {
            return HashMap::new();
        };
        let Ok(parsed) = serde_json::from_str::<Value>(&raw) else {
            return HashMap::new();
        };
        let mut out = HashMap::new();
        if let Some(details) = parsed.get("details").and_then(Value::as_object) {
            for (name, info) in details {
                let category = info.get("category").cloned().unwrap_or_else(|| json!("Other"));
                if let Some(regs) = info.get("registrations").and_then(Value::as_array) {
                    for reg in regs.iter().filter_map(Value::as_str) {
                        out.insert(
                            reg.trim().to_uppercase(),
                            json!({
                                "name": name,
                                "category": category,
                            }),
                        );
                    }
                }
            }
        }
        out
    })
}

pub fn category_color(category: &str) -> &'static str {
    match category {
        "USAF" | "Other Air Forces" | "United States Navy" | "UAV" | "Ukraine" => "yellow",
        "Flying Doctors" | "Aerial Firefighter" | "Coastguard" => "#32cd32",
        "Police Forces" | "Governments" | "Quango" => "blue",
        "Dictator Alert" | "Da Comrade" | "Oligarch" => "red",
        "Head of State" | "Royal Aircraft" | "Bizjets" => "#ff1493",
        _ => "purple",
    }
}

pub fn is_tracked(item: &Value) -> bool {
    let icao = item.get("icao24").and_then(Value::as_str).unwrap_or("");
    let reg = item.get("registration").and_then(Value::as_str).unwrap_or("");
    plane_alert_db().contains_key(icao) || tracked_names_db().contains_key(reg)
}

pub fn enrich_tracked(mut item: Value) -> Value {
    let icao = item.get("icao24").and_then(Value::as_str).unwrap_or("");
    if let Some(info) = plane_alert_db().get(icao) {
        item["alert_category"] = info.get("category").cloned().unwrap_or(Value::Null);
        item["alert_operator"] = info.get("operator").cloned().unwrap_or(Value::Null);
        item["alert_type"] = info.get("ac_type").cloned().unwrap_or(Value::Null);
        item["alert_tags"] = info.get("tags").cloned().unwrap_or(Value::Null);
        item["alert_link"] = info.get("link").cloned().unwrap_or(Value::Null);
        item["alert_color"] = json!(info
            .get("color")
            .and_then(Value::as_str)
            .unwrap_or_else(|| category_color(info.get("category").and_then(Value::as_str).unwrap_or(""))));
        if let Some(fleet) = info.get("potus_fleet") {
            item["potus_fleet"] = fleet.clone();
        }
    }
    let reg = item.get("registration").and_then(Value::as_str).unwrap_or("");
    if let Some(info) = tracked_names_db().get(reg) {
        item["tracked_name"] = info.get("name").cloned().unwrap_or(Value::Null);
        item["alert_category"] = info
            .get("category")
            .cloned()
            .or_else(|| item.get("alert_category").cloned())
            .unwrap_or(Value::Null);
        if item.get("alert_color").is_none() || item["alert_color"].is_null() {
            item["alert_color"] =
                json!(category_color(info.get("category").and_then(Value::as_str).unwrap_or("")));
        }
    }
    item["type"] = json!("tracked_flight");
    item
}

