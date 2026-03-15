use std::collections::HashMap;
use parking_lot::RwLock;
use chrono::{DateTime, Utc};
use serde_json::Value;

pub struct DataStore {
    pub data: RwLock<HashMap<String, Value>>,
    pub timestamps: RwLock<HashMap<String, DateTime<Utc>>>,
    pub viewport: RwLock<Option<Viewport>>,
    pub config: crate::config::Config,
    pub start_time: DateTime<Utc>,
    pub etag_fast: RwLock<String>,
    pub etag_slow: RwLock<String>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Viewport {
    pub south: f64,
    pub west: f64,
    pub north: f64,
    pub east: f64,
}

impl DataStore {
    pub fn new() -> Self {
        Self {
            data: RwLock::new(HashMap::new()),
            timestamps: RwLock::new(HashMap::new()),
            viewport: RwLock::new(None),
            config: crate::config::Config::from_env(),
            start_time: Utc::now(),
            etag_fast: RwLock::new(String::new()),
            etag_slow: RwLock::new(String::new()),
        }
    }

    pub fn set(&self, key: &str, value: Value) {
        self.data.write().insert(key.to_string(), value);
        self.timestamps.write().insert(key.to_string(), Utc::now());
    }

    pub fn get(&self, key: &str) -> Option<Value> {
        self.data.read().get(key).cloned()
    }

    pub fn get_timestamp(&self, key: &str) -> Option<DateTime<Utc>> {
        self.timestamps.read().get(key).cloned()
    }

    pub fn all_keys(&self) -> Vec<String> {
        self.data.read().keys().cloned().collect()
    }

    pub fn update_etag(&self, tier: &str) {
        let hash = format!("{:x}", md5_compute(Utc::now().to_rfc3339().as_bytes()));
        match tier {
            "fast" => *self.etag_fast.write() = hash,
            "slow" => *self.etag_slow.write() = hash,
            _ => {}
        }
    }

    pub fn get_etag(&self, tier: &str) -> String {
        match tier {
            "fast" => self.etag_fast.read().clone(),
            "slow" => self.etag_slow.read().clone(),
            _ => String::new(),
        }
    }
}

fn md5_compute(data: &[u8]) -> Md5Hex {
    use md5::{Md5, Digest};
    let result = Md5::digest(data);
    Md5Hex(result.into())
}

struct Md5Hex([u8; 16]);

impl std::fmt::LowerHex for Md5Hex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for b in &self.0 {
            write!(f, "{:02x}", b)?;
        }
        Ok(())
    }
}
