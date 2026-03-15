use crate::store::DataStore;
use serde_json::{json, Value};
use std::sync::Arc;

const DEFAULT_FEEDS: &[(&str, &str)] = &[
    ("NPR World", "https://feeds.npr.org/1004/rss.xml"),
    ("BBC World", "https://feeds.bbci.co.uk/news/world/rss.xml"),
    ("Al Jazeera", "https://www.aljazeera.com/xml/rss/all.xml"),
    ("NYT World", "https://rss.nytimes.com/services/xml/rss/nyt/World.xml"),
    ("GDACS", "https://www.gdacs.org/xml/rss.xml"),
    ("NHK World", "https://www3.nhk.or.jp/nhkworld/en/news/feeds/"),
    ("Reuters", "https://www.reutersagency.com/feed/"),
];

const RISK_KEYWORDS: &[(&str, u8)] = &[
    ("war", 9),
    ("missile", 9),
    ("nuclear", 10),
    ("attack", 8),
    ("bomb", 8),
    ("explosion", 7),
    ("invasion", 9),
    ("shooting", 7),
    ("crisis", 6),
    ("conflict", 6),
    ("military", 5),
    ("drone", 5),
    ("earthquake", 6),
    ("tsunami", 8),
    ("hurricane", 7),
    ("flood", 5),
    ("wildfire", 5),
    ("volcano", 6),
    ("pandemic", 7),
    ("outbreak", 5),
    ("sanctions", 4),
    ("coup", 8),
    ("protest", 4),
    ("riot", 5),
    ("hostage", 7),
    ("terrorism", 9),
    ("chemical", 8),
    ("cyber", 5),
];

pub async fn fetch(store: &Arc<DataStore>) -> anyhow::Result<()> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()?;

    let mut news_items: Vec<Value> = Vec::new();

    for (source, url) in DEFAULT_FEEDS {
        match client.get(*url).send().await {
            Ok(resp) if resp.status().is_success() => {
                if let Ok(body) = resp.bytes().await {
                    if let Ok(feed) = feed_rs::parser::parse(&body[..]) {
                        for entry in feed.entries.iter().take(10) {
                            let title = entry
                                .title
                                .as_ref()
                                .map(|t| t.content.clone())
                                .unwrap_or_default();

                            let risk = calculate_risk(&title);
                            let (lat, lng) = geolocate_by_keywords(&title);

                            let id = entry.id.clone();
                            let link = entry.links.first().map(|l| l.href.clone());
                            let published =
                                entry.published.or(entry.updated).map(|d| d.to_rfc3339());
                            let summary = entry.summary.as_ref().map(|s| s.content.clone());

                            news_items.push(json!({
                                "id": id,
                                "title": title,
                                "source": source,
                                "url": link,
                                "published": published,
                                "lat": lat,
                                "lng": lng,
                                "risk_score": risk,
                                "summary": summary,
                            }));
                        }
                    }
                }
            }
            _ => tracing::warn!("Failed to fetch feed: {}", source),
        }
    }

    // Sort by risk score descending
    news_items.sort_by(|a, b| {
        let ra = a.get("risk_score").and_then(|v| v.as_u64()).unwrap_or(0);
        let rb = b.get("risk_score").and_then(|v| v.as_u64()).unwrap_or(0);
        rb.cmp(&ra)
    });

    store.set("news", json!(news_items));
    store.update_etag("slow");
    tracing::info!("News updated: {} items", news_items.len());
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
        ("Ukraine", 48.38, 31.17),
        ("Russia", 55.75, 37.62),
        ("Gaza", 31.5, 34.47),
        ("Israel", 31.77, 35.23),
        ("Iran", 35.69, 51.39),
        ("China", 39.9, 116.4),
        ("Taiwan", 25.03, 121.57),
        ("North Korea", 39.02, 125.75),
        ("Syria", 33.51, 36.29),
        ("Iraq", 33.31, 44.37),
        ("Afghanistan", 34.53, 69.17),
        ("Yemen", 15.35, 44.21),
        ("Libya", 32.9, 13.18),
        ("Sudan", 15.6, 32.53),
        ("Somalia", 2.05, 45.34),
        ("Myanmar", 16.87, 96.2),
        ("Venezuela", 10.48, -66.9),
        ("Pakistan", 33.69, 73.04),
        ("India", 28.61, 77.21),
        ("Turkey", 39.93, 32.85),
        ("Egypt", 30.04, 31.24),
        ("Saudi Arabia", 24.71, 46.68),
        ("Lebanon", 33.89, 35.5),
        ("Japan", 35.68, 139.69),
        ("South Korea", 37.57, 126.98),
        ("Mexico", 19.43, -99.13),
        ("Brazil", -15.79, -47.88),
        ("Nigeria", 9.06, 7.49),
        ("Ethiopia", 9.02, 38.75),
        ("Congo", -4.32, 15.31),
        ("Kenya", -1.29, 36.82),
        ("South Africa", -33.93, 18.42),
        ("United States", 38.9, -77.04),
        ("United Kingdom", 51.51, -0.13),
        ("France", 48.86, 2.35),
        ("Germany", 52.52, 13.41),
        ("Poland", 52.23, 21.01),
        ("Romania", 44.43, 26.1),
        ("Philippines", 14.6, 120.98),
        ("Indonesia", -6.21, 106.85),
        ("Thailand", 13.76, 100.5),
        ("Vietnam", 21.03, 105.85),
        ("Washington", 38.9, -77.04),
        ("Moscow", 55.75, 37.62),
        ("Beijing", 39.9, 116.4),
        ("Kyiv", 50.45, 30.52),
        ("Tehran", 35.69, 51.39),
        ("Jerusalem", 31.77, 35.23),
        ("Taipei", 25.03, 121.57),
        ("Pyongyang", 39.02, 125.75),
        ("Seoul", 37.57, 126.98),
        ("Tokyo", 35.68, 139.69),
        ("London", 51.51, -0.13),
        ("Paris", 48.86, 2.35),
        ("Berlin", 52.52, 13.41),
        ("Kabul", 34.53, 69.17),
        ("Baghdad", 33.31, 44.37),
        ("Damascus", 33.51, 36.29),
        ("Ankara", 39.93, 32.85),
        ("Cairo", 30.04, 31.24),
        ("Riyadh", 24.71, 46.68),
        ("Beirut", 33.89, 35.5),
        ("Islamabad", 33.69, 73.04),
        ("New Delhi", 28.61, 77.21),
        ("Crimea", 44.95, 34.1),
        ("Donbas", 48.0, 37.8),
        ("Kherson", 46.64, 32.62),
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
