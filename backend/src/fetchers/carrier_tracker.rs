use serde_json::{json, Value};

const CARRIER_REGISTRY: &[(&str, &str, f64, f64, i64, &str, &str)] = &[
    ("CVN-68", "USS Nimitz (CVN-68)", 47.5535, -122.6400, 90, "Bremerton, WA", "https://en.wikipedia.org/wiki/USS_Nimitz"),
    ("CVN-69", "USS Dwight D. Eisenhower (CVN-69)", 36.9465, -76.3265, 0, "Norfolk, VA", "https://en.wikipedia.org/wiki/USS_Dwight_D._Eisenhower"),
    ("CVN-70", "USS Carl Vinson (CVN-70)", 32.6840, -117.1290, 180, "San Diego, CA", "https://en.wikipedia.org/wiki/USS_Carl_Vinson"),
    ("CVN-71", "USS Theodore Roosevelt (CVN-71)", 32.6885, -117.1280, 180, "San Diego, CA", "https://en.wikipedia.org/wiki/USS_Theodore_Roosevelt"),
    ("CVN-72", "USS Abraham Lincoln (CVN-72)", 20.0000, 64.0000, 90, "Arabian Sea", "https://en.wikipedia.org/wiki/USS_Abraham_Lincoln"),
    ("CVN-73", "USS George Washington (CVN-73)", 35.2830, 139.6700, 90, "Yokosuka", "https://en.wikipedia.org/wiki/USS_George_Washington"),
    ("CVN-75", "USS Harry S. Truman (CVN-75)", 36.0000, 15.0000, 120, "Mediterranean", "https://en.wikipedia.org/wiki/USS_Harry_S._Truman"),
    ("CVN-76", "USS Ronald Reagan (CVN-76)", 47.5580, -122.6360, 90, "Bremerton, WA", "https://en.wikipedia.org/wiki/USS_Ronald_Reagan"),
    ("CVN-77", "USS George H.W. Bush (CVN-77)", 36.5000, -74.0000, 110, "Atlantic", "https://en.wikipedia.org/wiki/USS_George_H.W._Bush"),
    ("CVN-78", "USS Gerald R. Ford (CVN-78)", 18.0000, 39.5000, 135, "Red Sea", "https://en.wikipedia.org/wiki/USS_Gerald_R._Ford"),
];

pub fn carrier_positions() -> Vec<Value> {
    CARRIER_REGISTRY
        .iter()
        .enumerate()
        .map(|(idx, (hull, name, lat, lng, heading, desc, wiki))| {
            json!({
                "mmsi": format!("338{:06}", idx + 1),
                "name": name,
                "hull": hull,
                "type": "carrier",
                "lat": lat,
                "lng": lng,
                "heading": heading,
                "country": "United States",
                "estimated": true,
                "source": "carrier_tracker",
                "desc": desc,
                "wiki": wiki,
            })
        })
        .collect()
}

