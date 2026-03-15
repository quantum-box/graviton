use crate::store::DataStore;
use serde_json::{json, Value};
use std::sync::Arc;

const RISK_KEYWORDS: &[(&str, u8)] = &[
    ("war", 9), ("missile", 9), ("nuclear", 10), ("attack", 8), ("bomb", 8), ("explosion", 7),
    ("invasion", 9), ("shooting", 7), ("crisis", 6), ("conflict", 6), ("military", 5), ("drone", 5),
    ("earthquake", 6), ("tsunami", 8), ("hurricane", 7), ("flood", 5), ("wildfire", 5),
    ("volcano", 6), ("pandemic", 7), ("outbreak", 5), ("sanctions", 4), ("coup", 8),
    ("protest", 4), ("riot", 5), ("hostage", 7), ("terrorism", 9), ("chemical", 8), ("cyber", 5),
];

pub async fn fetch(store: &Arc<DataStore>) -> anyhow::Result<()> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()?;

    let mut news_items: Vec<Value> = Vec::new();
    let feeds = crate::settings::load_news_feeds();
    for source in &feeds {
        match client.get(&source.url).send().await {
            Ok(resp) if resp.status().is_success() => {
                if let Ok(body) = resp.bytes().await {
                    if let Ok(feed) = feed_rs::parser::parse(&body[..]) {
                        for entry in feed.entries.iter().take(10) {
                            let title = entry.title.as_ref().map(|t| t.content.clone()).unwrap_or_default();
                            let risk = calculate_risk(&title);
                            let (lat, lng) = geolocate_by_keywords(&title);
                            news_items.push(json!({
                                "id": entry.id.clone(),
                                "title": title,
                                "source": source.name,
                                "url": entry.links.first().map(|l| l.href.clone()),
                                "published": entry.published.or(entry.updated).map(|d| d.to_rfc3339()),
                                "lat": lat,
                                "lng": lng,
                                "risk_score": risk,
                                "cluster_key": format!("{:.1}:{:.1}:{}", lat.unwrap_or(0.0), lng.unwrap_or(0.0), risk),
                                "summary": entry.summary.as_ref().map(|s| s.content.clone()),
                            }));
                        }
                    }
                }
            }
            _ => tracing::warn!("Failed to fetch feed: {}", source.name),
        }
    }
    news_items.sort_by(|a, b| {
        let ra = a.get("risk_score").and_then(|v| v.as_u64()).unwrap_or(0);
        let rb = b.get("risk_score").and_then(|v| v.as_u64()).unwrap_or(0);
        rb.cmp(&ra)
    });
    store.set("news", json!(news_items));
    store.update_etag("slow");
    Ok(())
}

fn calculate_risk(title: &str) -> u8 {
    let lower = title.to_lowercase();
    let mut max_risk = 1u8;
    for (kw, score) in RISK_KEYWORDS {
        if lower.contains(kw) {
            max_risk = max_risk.max(*score);
        }
    }
    max_risk
}

fn geolocate_by_keywords(title: &str) -> (Option<f64>, Option<f64>) {
    let locations: &[(&str, f64, f64)] = &[
        ("Ukraine", 48.38, 31.17), ("Russia", 55.75, 37.62), ("Gaza", 31.5, 34.47), ("Israel", 31.77, 35.23),
        ("Iran", 35.69, 51.39), ("China", 39.9, 116.4), ("Taiwan", 25.03, 121.57), ("North Korea", 39.02, 125.75),
        ("Syria", 33.51, 36.29), ("Iraq", 33.31, 44.37), ("Afghanistan", 34.53, 69.17), ("Yemen", 15.35, 44.21),
        ("Sudan", 15.6, 32.53), ("Myanmar", 16.87, 96.2), ("Pakistan", 33.69, 73.04), ("India", 28.61, 77.21),
        ("Turkey", 39.93, 32.85), ("Japan", 35.68, 139.69), ("South Korea", 37.57, 126.98), ("United States", 38.9, -77.04),
        ("United Kingdom", 51.51, -0.13), ("France", 48.86, 2.35), ("Germany", 52.52, 13.41), ("Poland", 52.23, 21.01),
        ("Kyiv", 50.45, 30.52), ("Tehran", 35.69, 51.39), ("Jerusalem", 31.77, 35.23), ("Taipei", 25.03, 121.57),
        ("Seoul", 37.57, 126.98), ("Tokyo", 35.68, 139.69), ("London", 51.51, -0.13), ("Paris", 48.86, 2.35),
        ("Berlin", 52.52, 13.41), ("Crimea", 44.95, 34.1), ("Donbas", 48.0, 37.8), ("Kherson", 46.64, 32.62),
        ("Zaporizhzhia", 47.84, 35.14),
    ];
    let lower = title.to_lowercase();
    for (name, lat, lng) in locations {
        if lower.contains(&name.to_lowercase()) {
            return (Some(*lat), Some(*lng));
        }
    }
    (None, None)
}
