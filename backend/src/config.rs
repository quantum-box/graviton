use std::env;

#[derive(Clone)]
pub struct Config {
    pub ais_api_key: Option<String>,
    pub opensky_client_id: Option<String>,
    pub opensky_client_secret: Option<String>,
    pub admin_key: Option<String>,
    pub cors_origins: Vec<String>,
    pub database_url: String,
    pub redis_url: String,
    pub jwt_secret: String,
    pub ais_stream_url: Option<String>,
    pub aircraft_stream_url: Option<String>,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            ais_api_key: read_secret("AIS_API_KEY"),
            opensky_client_id: read_secret("OPENSKY_CLIENT_ID"),
            opensky_client_secret: read_secret("OPENSKY_CLIENT_SECRET"),
            admin_key: read_secret("ADMIN_KEY"),
            cors_origins: env::var("CORS_ORIGINS")
                .unwrap_or_default()
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect(),
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgres://graviton:graviton@postgres:5432/graviton".to_string()),
            redis_url: env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://redis:6379".to_string()),
            jwt_secret: read_secret("JWT_SECRET").unwrap_or_else(|| "graviton-local-jwt-secret".to_string()),
            ais_stream_url: env::var("AIS_STREAM_URL").ok(),
            aircraft_stream_url: env::var("AIRCRAFT_STREAM_URL").ok(),
        }
    }

    pub fn env_status(&self) -> serde_json::Value {
        serde_json::json!({
            "ais_api_key": self.ais_api_key.is_some(),
            "opensky_client_id": self.opensky_client_id.is_some(),
            "opensky_client_secret": self.opensky_client_secret.is_some(),
            "admin_key": self.admin_key.is_some(),
            "cors_origins": self.cors_origins,
            "database_url": !self.database_url.is_empty(),
            "redis_url": !self.redis_url.is_empty(),
            "jwt_secret": !self.jwt_secret.is_empty(),
            "ais_stream_url": self.ais_stream_url.is_some(),
            "aircraft_stream_url": self.aircraft_stream_url.is_some(),
        })
    }
}

fn read_secret(key: &str) -> Option<String> {
    // Support Docker Swarm secrets via _FILE suffix
    if let Ok(path) = env::var(format!("{}_FILE", key)) {
        if let Ok(content) = std::fs::read_to_string(&path) {
            return Some(content.trim().to_string());
        }
    }
    env::var(key).ok()
}
