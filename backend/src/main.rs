use std::sync::Arc;
use axum::{
    Router,
    routing::{get, post},
    http::Method,
};
use tower_http::cors::{CorsLayer, Any};
use tower_http::compression::CompressionLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod settings;
mod store;
mod models;
mod fetchers;
mod routes;

pub type AppState = Arc<store::DataStore>;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| "graviton_backend=info,tower_http=info".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let store = Arc::new(store::DataStore::new());

    // Spawn background fetchers
    fetchers::spawn_all(store.clone());

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::OPTIONS])
        .allow_headers(Any);

    let app = Router::new()
        // Fast data (flights, ships, satellites)
        .route("/api/live-data/fast", get(routes::live_data_fast))
        // Slow data (news, earthquakes, weather, etc)
        .route("/api/live-data/slow", get(routes::live_data_slow))
        // Health check
        .route("/api/health", get(routes::health))
        // Flight route
        .route("/api/route/{callsign}", get(routes::flight_route))
        // Region dossier (right-click intel)
        .route("/api/region-dossier", get(routes::region_dossier))
        .route("/api/geocode/reverse", get(routes::reverse_geocode))
        .route("/api/geocode/search", get(routes::search_geocode))
        // Viewport update
        .route("/api/viewport", post(routes::update_viewport))
        // Radio endpoints
        .route("/api/radio/top", get(routes::radio_top))
        .route("/api/radio/openmhz/systems", get(routes::radio_openmhz_systems))
        .route("/api/radio/openmhz/calls/{sys_name}", get(routes::radio_openmhz_calls))
        .route("/api/radio/nearest", get(routes::radio_nearest))
        // Settings
        .route("/api/settings/api-keys", get(routes::get_api_keys).put(routes::update_api_keys))
        .route("/api/settings/news-feeds", get(routes::get_news_feeds).put(routes::update_news_feeds))
        .route("/api/settings/news-feeds/reset", post(routes::reset_news_feeds))
        // System
        .route("/api/refresh", post(routes::force_refresh))
        .route("/api/debug-latest", get(routes::debug_latest))
        // AIS feed ingestion
        .route("/api/ais/feed", post(routes::ais_feed))
        // Sentinel-2 search
        .route("/api/sentinel2/search", get(routes::sentinel_search))
        .layer(CompressionLayer::new())
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(store);

    let addr = "0.0.0.0:8000";
    tracing::info!("Graviton backend listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
