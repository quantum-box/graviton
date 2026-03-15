use std::env;

pub struct Config {
    pub ais_api_key: Option<String>,
    pub opensky_client_id: Option<String>,
    pub opensky_client_secret: Option<String>,
    pub admin_key: Option<String>,
    pub cors_origins: Vec<String>,
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
        }
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
