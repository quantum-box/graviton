use serde_json::{json, Value};

const UAV_TYPE_CODES: &[&str] = &["Q9", "R4", "TB2", "MALE", "HALE", "HERM", "HRON"];
const UAV_CALLSIGN_PREFIXES: &[&str] = &["FORTE", "GHAWK", "REAP", "BAMS", "UAV", "UAS"];
const UAV_MODEL_KEYWORDS: &[&str] = &[
    "RQ-", "MQ-", "RQ4", "MQ9", "MQ4", "MQ1", "REAPER", "GLOBALHAWK", "TRITON", "PREDATOR",
    "HERMES", "HERON", "BAYRAKTAR",
];

pub fn classify_aircraft(aircraft_type: &str, callsign: &str, operator: &str) -> &'static str {
    let t = aircraft_type.to_uppercase();
    let c = callsign.to_uppercase();
    let o = operator.to_uppercase();

    if is_uav(&t, &c) {
        return "uav";
    }
    if is_military_type(&t) || o.contains("AIR FORCE") || o.contains("NAVY") || o.contains("ARMY") {
        return "military_flight";
    }
    if is_private_jet_type(&t) {
        return "private_jet";
    }
    if c.len() >= 3 {
        let prefix = &c[..3];
        if [
            "AAL", "DAL", "UAL", "SWA", "BAW", "AFR", "DLH", "RYR", "EZY", "THY", "QFA", "SIA",
            "ANA", "JAL", "KAL", "CCA", "CSN", "CES",
        ]
        .contains(&prefix)
        {
            return "commercial_flight";
        }
    }
    "private_flight"
}

pub fn classify_military_type(aircraft_type: &str) -> &'static str {
    let t = aircraft_type.to_uppercase();
    if t.contains("H60") || t.contains("H47") || t.contains("UH") || t.contains("AH") || t.contains("H145") {
        "heli"
    } else if t.starts_with("KC") || t.contains("K35") || t.contains("K46") || t.contains("A33") || t.contains("TANK") {
        "tanker"
    } else if t.starts_with("RC") || t.starts_with("E3") || t.starts_with("P8") || t.starts_with("U2") {
        "recon"
    } else if t.starts_with("F") || t.starts_with("SU") || t.starts_with("MIG") || t.contains("A10") {
        "fighter"
    } else if t.starts_with("C") || t.contains("V22") {
        "cargo"
    } else {
        "default"
    }
}

pub fn is_military_type(t: &str) -> bool {
    t.starts_with("C1")
        || t.starts_with("C2")
        || t.starts_with("C5")
        || t.starts_with("F1")
        || t.starts_with("F2")
        || t.starts_with("F3")
        || t.starts_with("KC")
        || t.starts_with("E3")
        || t.starts_with("E6")
        || t.starts_with("E8")
        || t.starts_with("P8")
        || t.starts_with("RC")
        || t.starts_with("MQ")
        || t.starts_with("RQ")
        || t.contains("HAWK")
        || t.contains("HERC")
        || t.contains("GLOBEMASTER")
}

pub fn is_private_jet_type(t: &str) -> bool {
    [
        "GLF", "GLEX", "G550", "G650", "C680", "C56", "CL60", "LJ", "FA", "E55", "BD70", "GALX",
        "HDJT", "PC24", "SF50",
    ]
    .iter()
    .any(|needle| t.contains(needle))
}

pub fn is_uav(model: &str, callsign: &str) -> bool {
    let model_up = model.to_uppercase().replace(' ', "");
    let callsign_up = callsign.to_uppercase();
    UAV_TYPE_CODES.contains(&model_up.as_str())
        || UAV_CALLSIGN_PREFIXES.iter().any(|p| callsign_up.starts_with(p))
        || UAV_MODEL_KEYWORDS.iter().any(|kw| model_up.contains(kw))
}

pub fn uav_metadata(model: &str, callsign: &str) -> Value {
    let model_up = model.to_uppercase().replace(' ', "");
    let type_name = if model_up.contains("RQ4") || model_up.contains("GLOBALHAWK") || callsign.to_uppercase().starts_with("FORTE") {
        "HALE Surveillance"
    } else if model_up.contains("MQ4") || model_up.contains("TRITON") || callsign.to_uppercase().starts_with("BAMS") {
        "HALE Maritime Surveillance"
    } else if model_up.contains("MQ9") || model_up.contains("REAPER") || model_up.contains("BAYRAKTAR") {
        "MALE Strike/ISR"
    } else {
        "MALE ISR"
    };
    let wiki = if model_up.contains("RQ4") || model_up.contains("GLOBALHAWK") {
        "https://en.wikipedia.org/wiki/Northrop_Grumman_RQ-4_Global_Hawk"
    } else if model_up.contains("MQ4") || model_up.contains("TRITON") {
        "https://en.wikipedia.org/wiki/Northrop_Grumman_MQ-4C_Triton"
    } else if model_up.contains("MQ9") || model_up.contains("REAPER") {
        "https://en.wikipedia.org/wiki/General_Atomics_MQ-9_Reaper"
    } else if model_up.contains("MQ1") || model_up.contains("PREDATOR") {
        "https://en.wikipedia.org/wiki/General_Atomics_MQ-1_Predator"
    } else if model_up.contains("BAYRAKTAR") || model_up.contains("TB2") {
        "https://en.wikipedia.org/wiki/Bayraktar_TB2"
    } else {
        ""
    };
    json!({ "uav_type": type_name, "wiki": wiki })
}

