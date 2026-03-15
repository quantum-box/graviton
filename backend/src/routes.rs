use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashMap;

use crate::AppState;

#[derive(Deserialize)]
pub struct BboxQuery {
    pub s: Option<f64>,
    pub w: Option<f64>,
    pub n: Option<f64>,
    pub e: Option<f64>,
}

#[derive(Deserialize)]
pub struct LatLngQuery {
    pub lat: f64,
    pub lng: f64,
}

#[derive(Deserialize)]
pub struct SearchQuery {
    pub q: String,
}

pub async fn live_data_fast(
    State(store): State<AppState>,
    headers: HeaderMap,
    Query(_bbox): Query<BboxQuery>,
) -> impl IntoResponse {
    let etag = store.get_etag("fast");
    if let Some(if_none_match) = headers.get("if-none-match") {
        if let Ok(val) = if_none_match.to_str() {
            if val.trim_matches('"') == etag {
                return (StatusCode::NOT_MODIFIED, HeaderMap::new(), String::new()).into_response();
            }
        }
    }
    let data = json!({
        "last_updated": Utc::now().to_rfc3339(),
        "commercial_flights": store.get("commercial_flights").unwrap_or(json!([])),
        "private_flights": store.get("private_flights").unwrap_or(json!([])),
        "private_jets": store.get("private_jets").unwrap_or(json!([])),
        "military_flights": store.get("military_flights").unwrap_or(json!([])),
        "tracked_flights": store.get("tracked_flights").unwrap_or(json!([])),
        "uavs": store.get("uavs").unwrap_or(json!([])),
        "gps_jamming": store.get("gps_jamming").unwrap_or(json!([])),
        "ships": store.get("ships").unwrap_or(json!([])),
        "satellites": store.get("satellites").unwrap_or(json!([])),
    });
    let mut resp_headers = HeaderMap::new();
    resp_headers.insert("etag", format!("\"{}\"", etag).parse().unwrap());
    resp_headers.insert("cache-control", "no-cache".parse().unwrap());
    (StatusCode::OK, resp_headers, serde_json::to_string(&data).unwrap()).into_response()
}

pub async fn live_data_slow(
    State(store): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let etag = store.get_etag("slow");
    if let Some(if_none_match) = headers.get("if-none-match") {
        if let Ok(val) = if_none_match.to_str() {
            if val.trim_matches('"') == etag {
                return (StatusCode::NOT_MODIFIED, HeaderMap::new(), String::new()).into_response();
            }
        }
    }
    let data = json!({
        "last_updated": Utc::now().to_rfc3339(),
        "earthquakes": store.get("earthquakes").unwrap_or(json!([])),
        "news": store.get("news").unwrap_or(json!([])),
        "stocks": store.get("stocks").unwrap_or(json!([])),
        "oil": store.get("oil").unwrap_or(json!([])),
        "firms_fires": store.get("firms_fires").unwrap_or(json!([])),
        "gdelt": store.get("gdelt").unwrap_or(json!([])),
        "frontlines": store.get("frontlines").unwrap_or(json!({"type":"FeatureCollection","features":[]})),
        "liveuamap": store.get("liveuamap").unwrap_or(json!([])),
        "space_weather": store.get("space_weather").unwrap_or(json!({})),
        "weather": store.get("weather").unwrap_or(json!({})),
        "internet_outages": store.get("internet_outages").unwrap_or(json!([])),
        "kiwisdr": store.get("kiwisdr").unwrap_or(json!([])),
        "datacenters": store.get("datacenters").unwrap_or(json!([])),
        "cctv": store.get("cctv").unwrap_or(json!([])),
    });
    let mut resp_headers = HeaderMap::new();
    resp_headers.insert("etag", format!("\"{}\"", etag).parse().unwrap());
    (StatusCode::OK, resp_headers, serde_json::to_string(&data).unwrap()).into_response()
}

pub async fn health(State(store): State<AppState>) -> Json<Value> {
    let uptime = (Utc::now() - store.start_time).num_seconds();
    let mut sources = json!({});
    for key in store.all_keys() {
        if let Some(ts) = store.get_timestamp(&key) {
            sources[&key] = json!({
                "count": store.get(&key).and_then(|v| v.as_array().map(|a| a.len())).unwrap_or(0),
                "last_updated": ts.to_rfc3339(),
            });
        }
    }
    Json(json!({
        "status": "ok",
        "uptime_seconds": uptime,
        "sources": sources,
        "env": store.config.env_status(),
    }))
}

pub async fn flight_route(Path(callsign): Path<String>) -> Json<Value> {
    let client = reqwest::Client::new();
    match client
        .get(format!("https://api.adsb.lol/api/0/routeset?callsigns={}", callsign))
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => Json(resp.json::<Value>().await.unwrap_or_else(|_| json!({"error": "Route not found"}))),
        _ => Json(json!({"error": "Route not found"})),
    }
}

pub async fn region_dossier(Query(params): Query<LatLngQuery>) -> Json<Value> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .unwrap();
    Json(
        crate::fetchers::region_dossier::get_region_dossier(&client, params.lat, params.lng)
            .await
            .unwrap_or_else(|_| json!({"error": "Country data unavailable"})),
    )
}

pub async fn update_viewport(
    State(store): State<AppState>,
    Json(viewport): Json<crate::store::Viewport>,
) -> StatusCode {
    *store.viewport.write() = Some(viewport);
    StatusCode::OK
}

pub async fn reverse_geocode(Query(params): Query<LatLngQuery>) -> Json<Value> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .unwrap();
    let url = format!(
        "https://nominatim.openstreetmap.org/reverse?lat={}&lon={}&format=jsonv2",
        params.lat, params.lng
    );
    match client.get(url).header("User-Agent", "Graviton/1.0").send().await {
        Ok(resp) if resp.status().is_success() => Json(resp.json::<Value>().await.unwrap_or_else(|_| json!({}))),
        _ => Json(json!({})),
    }
}

pub async fn search_geocode(Query(params): Query<SearchQuery>) -> Json<Value> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .unwrap();
    let url = format!(
        "https://nominatim.openstreetmap.org/search?q={}&format=jsonv2&limit=8",
        urlencoding::encode(&params.q)
    );
    match client.get(url).header("User-Agent", "Graviton/1.0").send().await {
        Ok(resp) if resp.status().is_success() => Json(resp.json::<Value>().await.unwrap_or_else(|_| json!([]))),
        _ => Json(json!([])),
    }
}

pub async fn radio_top() -> Json<Value> {
    let client = reqwest::Client::new();
    Json(json!(crate::fetchers::radio_intercept::fetch_broadcastify_top(&client).await.unwrap_or_default()))
}

pub async fn radio_openmhz_systems() -> Json<Value> {
    let client = reqwest::Client::new();
    Json(crate::fetchers::radio_intercept::openmhz_systems(&client).await.unwrap_or_else(|_| json!([])))
}

pub async fn radio_openmhz_calls(Path(sys_name): Path<String>) -> Json<Value> {
    let client = reqwest::Client::new();
    Json(crate::fetchers::radio_intercept::openmhz_calls(&client, &sys_name).await.unwrap_or_else(|_| json!([])))
}

pub async fn radio_nearest(Query(params): Query<LatLngQuery>) -> Json<Value> {
    let client = reqwest::Client::new();
    Json(json!(crate::fetchers::radio_intercept::nearest_systems(&client, params.lat, params.lng, 5).await.unwrap_or_default()))
}

pub async fn get_api_keys(State(store): State<AppState>, headers: HeaderMap) -> impl IntoResponse {
    if !check_admin(&store, &headers) {
        return (StatusCode::UNAUTHORIZED, Json(json!({"error": "Unauthorized"}))).into_response();
    }
    Json(json!({
        "registry": crate::settings::get_api_registry(&store.config),
        "ais_api_key": store.config.ais_api_key.as_ref().map(|_| "***configured***"),
        "opensky_client_id": store.config.opensky_client_id.as_ref().map(|_| "***configured***"),
    }))
    .into_response()
}

pub async fn update_api_keys(
    State(store): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> impl IntoResponse {
    if !check_admin(&store, &headers) {
        return StatusCode::UNAUTHORIZED;
    }
    let values = body
        .as_object()
        .map(|obj| {
            obj.iter()
                .filter_map(|(k, v)| v.as_str().map(|s| (k.to_string(), s.to_string())))
                .collect::<HashMap<_, _>>()
        })
        .unwrap_or_default();
    if crate::settings::update_api_keys(&values).is_ok() {
        StatusCode::OK
    } else {
        StatusCode::BAD_REQUEST
    }
}

pub async fn get_news_feeds() -> Json<Value> {
    Json(json!({ "feeds": crate::settings::load_news_feeds() }))
}

pub async fn update_news_feeds(
    State(store): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> impl IntoResponse {
    if !check_admin(&store, &headers) {
        return StatusCode::UNAUTHORIZED;
    }
    let feeds = body
        .get("feeds")
        .cloned()
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default();
    if crate::settings::save_news_feeds(feeds).is_ok() {
        StatusCode::OK
    } else {
        StatusCode::BAD_REQUEST
    }
}

pub async fn reset_news_feeds() -> StatusCode {
    if crate::settings::reset_news_feeds().is_ok() {
        StatusCode::OK
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

pub async fn force_refresh(State(_store): State<AppState>) -> StatusCode {
    StatusCode::OK
}

pub async fn debug_latest(State(store): State<AppState>) -> Json<Value> {
    let mut info = json!({});
    for key in store.all_keys() {
        let count = store.get(&key).and_then(|v| v.as_array().map(|a| a.len())).unwrap_or(0);
        info[&key] = json!(count);
    }
    Json(info)
}

pub async fn ais_feed(State(store): State<AppState>, Json(body): Json<Value>) -> StatusCode {
    let mut ships = Vec::new();
    let items = body
        .get("messages")
        .and_then(|v| v.as_array())
        .cloned()
        .or_else(|| body.as_array().cloned())
        .unwrap_or_default();
    for item in items {
        let Some(mmsi) = item.get("mmsi").and_then(|v| v.as_i64().or_else(|| v.as_str()?.parse().ok())) else {
            continue;
        };
        let lat = item.get("lat").or_else(|| item.get("latitude")).and_then(|v| v.as_f64().or_else(|| v.as_str()?.parse().ok()));
        let lng = item
            .get("lon")
            .or_else(|| item.get("lng"))
            .or_else(|| item.get("longitude"))
            .and_then(|v| v.as_f64().or_else(|| v.as_str()?.parse().ok()));
        let (Some(lat), Some(lng)) = (lat, lng) else { continue };
        ships.push(json!({
            "mmsi": mmsi.to_string(),
            "name": item.get("shipname").or_else(|| item.get("name")).cloned().unwrap_or(Value::Null),
            "type": classify_ais_type(item.get("shiptype").and_then(|v| v.as_u64()).unwrap_or(0)),
            "lat": lat,
            "lng": lng,
            "heading": item.get("heading").cloned().unwrap_or(Value::Null),
            "sog": item.get("speed").or_else(|| item.get("sog")).cloned().unwrap_or(Value::Null),
            "cog": item.get("course").or_else(|| item.get("cog")).cloned().unwrap_or(Value::Null),
            "destination": item.get("destination").cloned().unwrap_or(Value::Null),
            "country": item.get("country").cloned().unwrap_or(Value::Null),
            "source": "ais_ingest",
        }));
    }
    store.set("ais_feed_snapshot", json!(ships));
    store.update_etag("fast");
    StatusCode::OK
}

pub async fn sentinel_search(Query(params): Query<LatLngQuery>) -> Json<Value> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .unwrap();
    Json(
        crate::fetchers::sentinel::search(&client, params.lat, params.lng)
            .await
            .unwrap_or_else(|_| json!({"features": []})),
    )
}

fn check_admin(store: &AppState, headers: &HeaderMap) -> bool {
    match &store.config.admin_key {
        Some(key) => headers
            .get("x-admin-key")
            .and_then(|v| v.to_str().ok())
            .map(|v| v == key)
            .unwrap_or(false),
        None => true,
    }
}

fn classify_ais_type(code: u64) -> &'static str {
    match code {
        80..=89 => "tanker",
        70..=79 => "cargo",
        60..=69 => "passenger",
        35 => "military_vessel",
        36 | 37 => "yacht",
        _ => "other",
    }
}
