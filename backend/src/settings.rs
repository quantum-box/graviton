use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{collections::HashMap, fs, path::PathBuf};

const DEFAULT_NEWS_FEEDS: &[(&str, &str, i64)] = &[
    ("NPR", "https://feeds.npr.org/1004/rss.xml", 4),
    ("BBC", "http://feeds.bbci.co.uk/news/world/rss.xml", 3),
    ("AlJazeera", "https://www.aljazeera.com/xml/rss/all.xml", 2),
    ("NYT", "https://rss.nytimes.com/services/xml/rss/nyt/World.xml", 1),
    ("GDACS", "https://www.gdacs.org/xml/rss.xml", 5),
    ("NHK", "https://www3.nhk.or.jp/nhkworld/rss/world.xml", 3),
    ("CNA", "https://www.channelnewsasia.com/rssfeed/8395986", 3),
    ("Mercopress", "https://en.mercopress.com/rss/", 3),
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsFeed {
    pub name: String,
    pub url: String,
    pub weight: i64,
}

fn config_dir() -> PathBuf {
    PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/config"))
}

fn news_feed_path() -> PathBuf {
    config_dir().join("news_feeds.json")
}

fn env_path() -> PathBuf {
    PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/.env"))
}

pub fn default_news_feeds() -> Vec<NewsFeed> {
    DEFAULT_NEWS_FEEDS
        .iter()
        .map(|(name, url, weight)| NewsFeed {
            name: (*name).to_string(),
            url: (*url).to_string(),
            weight: *weight,
        })
        .collect()
}

pub fn load_news_feeds() -> Vec<NewsFeed> {
    let path = news_feed_path();
    fs::read_to_string(&path)
        .ok()
        .and_then(|raw| serde_json::from_str::<serde_json::Value>(&raw).ok())
        .and_then(|data| {
            data.get("feeds")
                .cloned()
                .or(Some(data))
                .and_then(|feeds| serde_json::from_value(feeds).ok())
        })
        .filter(|feeds: &Vec<NewsFeed>| !feeds.is_empty())
        .unwrap_or_else(default_news_feeds)
}

pub fn save_news_feeds(feeds: Vec<NewsFeed>) -> anyhow::Result<()> {
    fs::create_dir_all(config_dir())?;
    fs::write(
        news_feed_path(),
        serde_json::to_vec_pretty(&json!({ "feeds": feeds }))?,
    )?;
    Ok(())
}

pub fn reset_news_feeds() -> anyhow::Result<Vec<NewsFeed>> {
    let feeds = default_news_feeds();
    save_news_feeds(feeds.clone())?;
    Ok(feeds)
}

pub fn get_api_registry(config: &crate::config::Config) -> serde_json::Value {
    json!([
        {
            "id": "opensky_client_id",
            "env_key": "OPENSKY_CLIENT_ID",
            "name": "OpenSky Network — Client ID",
            "category": "Aviation",
            "required": true,
            "is_set": config.opensky_client_id.is_some(),
        },
        {
            "id": "opensky_client_secret",
            "env_key": "OPENSKY_CLIENT_SECRET",
            "name": "OpenSky Network — Client Secret",
            "category": "Aviation",
            "required": true,
            "is_set": config.opensky_client_secret.is_some(),
        },
        {
            "id": "ais_api_key",
            "env_key": "AIS_API_KEY",
            "name": "AIS Stream",
            "category": "Maritime",
            "required": true,
            "is_set": config.ais_api_key.is_some(),
        }
    ])
}

pub fn update_api_keys(new_values: &HashMap<String, String>) -> anyhow::Result<()> {
    let env_file = env_path();
    let mut lines: Vec<String> = fs::read_to_string(&env_file)
        .unwrap_or_default()
        .lines()
        .map(|line| line.to_string())
        .collect();
    for (key, value) in new_values {
        let mut updated = false;
        for line in &mut lines {
            if line.starts_with(&format!("{}=", key)) {
                *line = format!("{}={}", key, value);
                updated = true;
                break;
            }
        }
        if !updated {
            lines.push(format!("{}={}", key, value));
        }
    }
    fs::write(env_file, format!("{}\n", lines.join("\n")))?;
    Ok(())
}
