use std::{collections::HashMap, time::Duration};

use anyhow::Context;
use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::{postgres::PgPoolOptions, PgPool, Row};
use tokio::sync::broadcast;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Viewport {
    pub south: f64,
    pub west: f64,
    pub north: f64,
    pub east: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AlertEvent {
    pub id: String,
    pub severity: String,
    pub title: String,
    pub message: String,
    pub source: String,
    pub entity_id: Option<String>,
    pub entity_type: Option<String>,
    pub lat: Option<f64>,
    pub lng: Option<f64>,
    pub created_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsEvent {
    Snapshot { fast: Value, slow: Value },
    Dataset { key: String, payload: Value },
    Alert { alert: AlertEvent },
    Simulation { payload: Value },
}

pub struct DataStore {
    pub data: RwLock<HashMap<String, Value>>,
    pub timestamps: RwLock<HashMap<String, DateTime<Utc>>>,
    pub viewport: RwLock<Option<Viewport>>,
    pub config: crate::config::Config,
    pub start_time: DateTime<Utc>,
    pub etag_fast: RwLock<String>,
    pub etag_slow: RwLock<String>,
    pub db: PgPool,
    pub redis_client: redis::Client,
    pub broadcaster: broadcast::Sender<WsEvent>,
    pub latest_alerts: RwLock<Vec<AlertEvent>>,
}

impl DataStore {
    pub async fn new() -> anyhow::Result<Self> {
        let config = crate::config::Config::from_env();
        let db = PgPoolOptions::new()
            .max_connections(10)
            .acquire_timeout(Duration::from_secs(10))
            .connect(&config.database_url)
            .await
            .with_context(|| "failed to connect postgres")?;
        initialize_schema(&db).await?;
        let redis_client = redis::Client::open(config.redis_url.clone()).with_context(|| "failed to create redis client")?;
        let _: redis::aio::ConnectionManager = redis_client
            .get_connection_manager()
            .await
            .with_context(|| "failed to connect redis")?;
        let (broadcaster, _) = broadcast::channel(512);
        Ok(Self {
            data: RwLock::new(HashMap::new()),
            timestamps: RwLock::new(HashMap::new()),
            viewport: RwLock::new(None),
            config,
            start_time: Utc::now(),
            etag_fast: RwLock::new(String::new()),
            etag_slow: RwLock::new(String::new()),
            db,
            redis_client,
            broadcaster,
            latest_alerts: RwLock::new(Vec::new()),
        })
    }

    pub fn set(&self, key: &str, value: Value) {
        self.data.write().insert(key.to_string(), value.clone());
        self.timestamps.write().insert(key.to_string(), Utc::now());
        let _ = self.broadcaster.send(WsEvent::Dataset {
            key: key.to_string(),
            payload: value,
        });
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

    pub fn fast_snapshot(&self) -> Value {
        json!({
            "last_updated": Utc::now().to_rfc3339(),
            "commercial_flights": self.get("commercial_flights").unwrap_or(json!([])),
            "private_flights": self.get("private_flights").unwrap_or(json!([])),
            "private_jets": self.get("private_jets").unwrap_or(json!([])),
            "military_flights": self.get("military_flights").unwrap_or(json!([])),
            "tracked_flights": self.get("tracked_flights").unwrap_or(json!([])),
            "uavs": self.get("uavs").unwrap_or(json!([])),
            "gps_jamming": self.get("gps_jamming").unwrap_or(json!([])),
            "ships": self.get("ships").unwrap_or(json!([])),
            "satellites": self.get("satellites").unwrap_or(json!([])),
            "fused_objects": self.get("fused_objects").unwrap_or(json!([])),
            "simulation_markers": self.get("simulation_markers").unwrap_or(json!([])),
            "alerts": json!(self.latest_alerts.read().clone()),
        })
    }

    pub fn slow_snapshot(&self) -> Value {
        json!({
            "last_updated": Utc::now().to_rfc3339(),
            "earthquakes": self.get("earthquakes").unwrap_or(json!([])),
            "news": self.get("news").unwrap_or(json!([])),
            "stocks": self.get("stocks").unwrap_or(json!([])),
            "oil": self.get("oil").unwrap_or(json!([])),
            "firms_fires": self.get("firms_fires").unwrap_or(json!([])),
            "gdelt": self.get("gdelt").unwrap_or(json!([])),
            "frontlines": self.get("frontlines").unwrap_or(json!({"type":"FeatureCollection","features":[]})),
            "liveuamap": self.get("liveuamap").unwrap_or(json!([])),
            "space_weather": self.get("space_weather").unwrap_or(json!({})),
            "weather": self.get("weather").unwrap_or(json!({})),
            "internet_outages": self.get("internet_outages").unwrap_or(json!([])),
            "kiwisdr": self.get("kiwisdr").unwrap_or(json!([])),
            "datacenters": self.get("datacenters").unwrap_or(json!([])),
            "cctv": self.get("cctv").unwrap_or(json!([])),
        })
    }

    pub fn subscribe(&self) -> broadcast::Receiver<WsEvent> {
        self.broadcaster.subscribe()
    }

    pub async fn cache_get_json(&self, key: &str) -> anyhow::Result<Option<Value>> {
        let mut conn = self.redis_client.get_multiplexed_async_connection().await?;
        let payload: Option<String> = conn.get(key).await?;
        Ok(payload.and_then(|item| serde_json::from_str(&item).ok()))
    }

    pub async fn cache_set_json(&self, key: &str, value: &Value, ttl_seconds: u64) -> anyhow::Result<()> {
        let mut conn = self.redis_client.get_multiplexed_async_connection().await?;
        let serialized = serde_json::to_string(value)?;
        let _: () = conn.set_ex(key, serialized, ttl_seconds).await?;
        Ok(())
    }

    pub async fn persist_positions(&self, object_type: &str, source: &str, items: &[Value]) -> anyhow::Result<()> {
        for item in items {
            let Some(lat) = item.get("lat").and_then(Value::as_f64) else { continue };
            let Some(lng) = item.get("lng").and_then(Value::as_f64) else { continue };
            let object_id = item
                .get("icao24")
                .or_else(|| item.get("mmsi"))
                .or_else(|| item.get("id"))
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string();
            if object_id.is_empty() {
                continue;
            }
            let speed = item
                .get("speed_knots")
                .or_else(|| item.get("speed"))
                .and_then(Value::as_f64);
            let heading = item.get("heading").and_then(Value::as_f64);
            sqlx::query(
                r#"
                INSERT INTO object_history (object_type, object_id, source, ts, lat, lng, speed, heading, metadata)
                VALUES ($1, $2, $3, NOW(), $4, $5, $6, $7, $8)
                "#,
            )
            .bind(object_type)
            .bind(&object_id)
            .bind(source)
            .bind(lat)
            .bind(lng)
            .bind(speed)
            .bind(heading)
            .bind(sqlx::types::Json(item.clone()))
            .execute(&self.db)
            .await?;
        }
        Ok(())
    }

    pub async fn emit_alert(&self, alert: AlertEvent) {
        {
            let mut alerts = self.latest_alerts.write();
            alerts.insert(0, alert.clone());
            alerts.truncate(50);
        }
        let _ = self.broadcaster.send(WsEvent::Alert { alert: alert.clone() });
        let _ = sqlx::query(
            "INSERT INTO alert_events (id, severity, title, message, source, entity_id, entity_type, lat, lng, created_at) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,NOW()) ON CONFLICT (id) DO NOTHING",
        )
        .bind(&alert.id)
        .bind(&alert.severity)
        .bind(&alert.title)
        .bind(&alert.message)
        .bind(&alert.source)
        .bind(&alert.entity_id)
        .bind(&alert.entity_type)
        .bind(alert.lat)
        .bind(alert.lng)
        .execute(&self.db)
        .await;
        send_discord_webhooks(&self.db, &alert).await;
    }

    pub async fn load_recent_alerts(&self) -> anyhow::Result<()> {
        let rows = sqlx::query(
            "SELECT id, severity, title, message, source, entity_id, entity_type, lat, lng, created_at FROM alert_events ORDER BY created_at DESC LIMIT 50",
        )
        .fetch_all(&self.db)
        .await?;
        let alerts = rows
            .into_iter()
            .map(|row| AlertEvent {
                id: row.get("id"),
                severity: row.get("severity"),
                title: row.get("title"),
                message: row.get("message"),
                source: row.get("source"),
                entity_id: row.try_get("entity_id").ok(),
                entity_type: row.try_get("entity_type").ok(),
                lat: row.try_get("lat").ok(),
                lng: row.try_get("lng").ok(),
                created_at: row.get::<DateTime<Utc>, _>("created_at").to_rfc3339(),
            })
            .collect::<Vec<_>>();
        *self.latest_alerts.write() = alerts;
        Ok(())
    }
}

async fn send_discord_webhooks(db: &PgPool, alert: &AlertEvent) {
    let rows = match sqlx::query("SELECT webhook_url FROM webhook_settings WHERE webhook_url IS NOT NULL")
        .fetch_all(db)
        .await
    {
        Ok(rows) => rows,
        Err(_) => return,
    };
    let client = reqwest::Client::new();
    for row in rows {
        let webhook_url: String = row.get("webhook_url");
        let _ = client
            .post(webhook_url)
            .json(&json!({
                "content": format!("[{}] {} - {}", alert.severity.to_uppercase(), alert.title, alert.message),
            }))
            .send()
            .await;
    }
}

async fn initialize_schema(db: &PgPool) -> anyhow::Result<()> {
    sqlx::query("CREATE EXTENSION IF NOT EXISTS timescaledb").execute(db).await.ok();
    for statement in [
        r#"CREATE TABLE IF NOT EXISTS users (
            id UUID PRIMARY KEY,
            email TEXT UNIQUE NOT NULL,
            password_hash TEXT NOT NULL,
            display_name TEXT NOT NULL,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )"#,
        r#"CREATE TABLE IF NOT EXISTS object_history (
            object_type TEXT NOT NULL,
            object_id TEXT NOT NULL,
            source TEXT NOT NULL,
            ts TIMESTAMPTZ NOT NULL,
            lat DOUBLE PRECISION NOT NULL,
            lng DOUBLE PRECISION NOT NULL,
            speed DOUBLE PRECISION,
            heading DOUBLE PRECISION,
            metadata JSONB NOT NULL DEFAULT '{}'::jsonb
        )"#,
        r#"CREATE INDEX IF NOT EXISTS idx_object_history_lookup ON object_history (object_type, object_id, ts DESC)"#,
        r#"CREATE TABLE IF NOT EXISTS shared_objects (
            id UUID PRIMARY KEY,
            user_id UUID,
            kind TEXT NOT NULL,
            title TEXT NOT NULL,
            payload JSONB NOT NULL,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )"#,
        r#"CREATE TABLE IF NOT EXISTS webhook_settings (
            user_id UUID PRIMARY KEY,
            webhook_url TEXT
        )"#,
        r#"CREATE TABLE IF NOT EXISTS alert_events (
            id TEXT PRIMARY KEY,
            severity TEXT NOT NULL,
            title TEXT NOT NULL,
            message TEXT NOT NULL,
            source TEXT NOT NULL,
            entity_id TEXT,
            entity_type TEXT,
            lat DOUBLE PRECISION,
            lng DOUBLE PRECISION,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )"#,
        r#"CREATE TABLE IF NOT EXISTS missions (
            id UUID PRIMARY KEY,
            user_id UUID,
            title TEXT NOT NULL,
            status TEXT NOT NULL,
            payload JSONB NOT NULL,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )"#,
        r#"CREATE TABLE IF NOT EXISTS watchlist (
            id UUID PRIMARY KEY,
            user_id UUID,
            target_id TEXT NOT NULL,
            target_type TEXT NOT NULL,
            note TEXT,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )"#,
    ] {
        sqlx::query(statement).execute(db).await?;
    }
    sqlx::query("SELECT create_hypertable('object_history', 'ts', if_not_exists => TRUE)")
        .execute(db)
        .await
        .ok();
    Ok(())
}

fn md5_compute(data: &[u8]) -> Md5Hex {
    use md5::{Digest, Md5};
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
