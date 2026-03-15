use crate::store::DataStore;
use serde_json::{json, Value};
use std::sync::Arc;

const CARRIER_ESTIMATES: &[(&str, &str, f64, f64)] = &[
    ("CVN-68", "USS Nimitz", 21.3, -157.9),
    ("CVN-69", "USS Dwight D. Eisenhower", 25.0, 60.0),
    ("CVN-70", "USS Carl Vinson", 35.3, 139.7),
    ("CVN-71", "USS Theodore Roosevelt", 32.7, -117.2),
    ("CVN-72", "USS Abraham Lincoln", 21.0, 65.0),
    ("CVN-73", "USS George Washington", 35.3, 139.7),
    ("CVN-74", "USS John C. Stennis", 47.6, -122.3),
    ("CVN-75", "USS Harry S. Truman", 36.9, 0.0),
    ("CVN-76", "USS Ronald Reagan", 33.0, 132.0),
    ("CVN-77", "USS George H.W. Bush", 36.8, -76.3),
    ("CVN-78", "USS Gerald R. Ford", 36.8, -76.3),
];

pub async fn fetch(store: &Arc<DataStore>) -> anyhow::Result<()> {
    let mut ships: Vec<Value> = Vec::new();

    // Add carrier estimated positions
    for (hull, name, lat, lng) in CARRIER_ESTIMATES {
        ships.push(json!({
            "mmsi": hull,
            "name": name,
            "lat": lat,
            "lng": lng,
            "ship_type": "military",
            "heading": 0,
            "speed": 0,
            "country": "US",
        }));
    }

    store.set("ships", json!(ships));
    store.update_etag("fast");
    tracing::info!("Ships updated: {} total", ships.len());
    Ok(())
}

pub fn _classify_ship_type(ship_type: u32) -> &'static str {
    match ship_type {
        70..=79 => "cargo",
        80..=89 => "tanker",
        60..=69 => "passenger",
        30..=39 => "fishing",
        50..=59 => "special",
        35 => "military",
        _ => "other",
    }
}

pub fn _mmsi_to_country(mmsi: &str) -> Option<&'static str> {
    let mid: u32 = mmsi.get(0..3)?.parse().ok()?;
    Some(match mid {
        201..=212 => "Greece",
        211 => "Germany",
        215 => "Malta",
        219..=220 => "Denmark",
        224..=226 => "Spain",
        227..=228 => "France",
        230 => "Finland",
        231 => "Faroe Islands",
        232..=235 => "United Kingdom",
        236 => "Gibraltar",
        237 | 240 | 241 => "Greece",
        242 => "Morocco",
        243..=244 => "Hungary",
        245..=246 => "Netherlands",
        247..=249 => "Italy",
        250 => "Ireland",
        255 => "Portugal",
        256 => "Malta",
        257..=259 => "Norway",
        261 => "Poland",
        263 => "Portugal",
        265..=267 => "Sweden",
        268..=270 => "Czech Republic",
        271 => "Turkey",
        272 => "Ukraine",
        273 => "Russia",
        303 => "Alaska (US)",
        304..=307 => "Antigua and Barbuda",
        308..=309 | 311 => "Bahamas",
        310 => "Bermuda",
        312 => "Belize",
        316 => "Canada",
        319 => "Cayman Islands",
        338..=339 | 366..=369 => "United States",
        341 | 345..=347 => "Mexico",
        351..=357 | 370..=379 => "Panama",
        401..=403 => "Afghanistan",
        412..=413 => "China",
        416 => "Taiwan",
        417 | 477 => "Sri Lanka",
        419 | 459 => "India",
        422 => "Iran",
        431..=432 => "Japan",
        440..=441 => "South Korea",
        443 => "Palestine",
        445 => "North Korea",
        447 => "Kuwait",
        450 => "Lebanon",
        451 => "Kyrgyzstan",
        453 => "Macao",
        455..=457 => "Maldives",
        461 => "Mongolia",
        463 => "Nepal",
        466..=468 => "Oman",
        470 => "Pakistan",
        472 => "Qatar",
        473 => "Saudi Arabia",
        475 => "Singapore",
        478 => "Syria",
        501..=503 => "France (overseas)",
        506..=508 | 523 | 605 => "Mozambique",
        509 => "Mauritius",
        510 => "Micronesia",
        511..=512 => "Marshall Islands",
        514 => "Madagascar",
        515..=519 => "Comoros",
        520..=521 | 557 => "Zimbabwe",
        525 => "Namibia",
        529 => "Nigeria",
        533 | 559 | 601..=603 => "South Africa",
        536..=538 => "Tanzania",
        540..=542 => "Togo",
        548..=550 => "Uganda",
        553..=555 => "Zambia",
        561..=563 => "Kenya",
        564..=567 => "Democratic Republic of the Congo",
        _ => "Unknown",
    })
}
