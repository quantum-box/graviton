use axum::{
    extract::{State, Path, Query},
    http::{HeaderMap, StatusCode},
    Json,
    response::IntoResponse,
};
use serde::Deserialize;
use serde_json::{json, Value};
use crate::AppState;
use chrono::Utc;

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

// Fast-tier data: flights, ships, satellites
pub async fn live_data_fast(
    State(store): State<AppState>,
    headers: HeaderMap,
    Query(_bbox): Query<BboxQuery>,
) -> impl IntoResponse {
    let etag = store.get_etag("fast");

    // ETag check for 304
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

// Slow-tier data: news, earthquakes, weather, etc
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
    let keys = store.all_keys();
    let mut sources = json!({});
    for key in &keys {
        if let Some(ts) = store.get_timestamp(key) {
            sources[key] = json!({
                "count": store.get(key).and_then(|v| v.as_array().map(|a| a.len())).unwrap_or(0),
                "last_updated": ts.to_rfc3339(),
            });
        }
    }

    Json(json!({
        "status": "ok",
        "uptime_seconds": uptime,
        "sources": sources,
    }))
}

pub async fn flight_route(
    Path(callsign): Path<String>,
) -> Json<Value> {
    let client = reqwest::Client::new();
    match client.get(format!("https://api.adsb.lol/api/0/routeset?callsigns={}", callsign))
        .send().await
    {
        Ok(resp) if resp.status().is_success() => {
            if let Ok(data) = resp.json::<Value>().await {
                return Json(data);
            }
        }
        _ => {}
    }
    Json(json!({"error": "Route not found"}))
}

pub async fn region_dossier(
    Query(params): Query<LatLngQuery>,
) -> Json<Value> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .unwrap();

    // Reverse geocode to country
    let nominatim_url = format!(
        "https://nominatim.openstreetmap.org/reverse?lat={}&lon={}&format=json&zoom=3",
        params.lat, params.lng
    );

    let country_code = match client.get(&nominatim_url)
        .header("User-Agent", "Graviton/1.0")
        .send().await
    {
        Ok(resp) if resp.status().is_success() => {
            resp.json::<Value>().await.ok()
                .and_then(|d| d.get("address")
                    .and_then(|a| a.get("country_code"))
                    .and_then(|c| c.as_str())
                    .map(|s| s.to_uppercase()))
        }
        _ => None,
    };

    let Some(cc) = country_code else {
        return Json(json!({"error": "Could not determine country"}));
    };

    // Fetch country details from RestCountries
    match client.get(format!("https://restcountries.com/v3.1/alpha/{}", cc))
        .send().await
    {
        Ok(resp) if resp.status().is_success() => {
            if let Ok(data) = resp.json::<Vec<Value>>().await {
                if let Some(country) = data.first() {
                    let name = country.get("name")
                        .and_then(|n| n.get("common"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("Unknown");
                    let flag = country.get("flag").and_then(|v| v.as_str());
                    let capital = country.get("capital")
                        .and_then(|c| c.as_array())
                        .and_then(|a| a.first())
                        .and_then(|v| v.as_str());
                    let population = country.get("population").and_then(|v| v.as_u64());
                    let region = country.get("region").and_then(|v| v.as_str());
                    let languages: Vec<String> = country.get("languages")
                        .and_then(|l| l.as_object())
                        .map(|m| m.values().filter_map(|v| v.as_str().map(String::from)).collect())
                        .unwrap_or_default();
                    let currencies: Vec<String> = country.get("currencies")
                        .and_then(|c| c.as_object())
                        .map(|m| m.values().filter_map(|v| v.get("name").and_then(|n| n.as_str()).map(String::from)).collect())
                        .unwrap_or_default();

                    return Json(json!({
                        "country_name": name,
                        "flag": flag,
                        "capital": capital,
                        "population": population,
                        "region": region,
                        "languages": languages,
                        "currencies": currencies,
                    }));
                }
            }
        }
        _ => {}
    }

    Json(json!({"error": "Country data unavailable"}))
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
    match client
        .get(url)
        .header("User-Agent", "Graviton/1.0")
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => {
            if let Ok(data) = resp.json::<Value>().await {
                return Json(data);
            }
        }
        _ => {}
    }
    Json(json!({}))
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
    match client
        .get(url)
        .header("User-Agent", "Graviton/1.0")
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => {
            if let Ok(data) = resp.json::<Value>().await {
                return Json(data);
            }
        }
        _ => {}
    }
    Json(json!([]))
}

pub async fn radio_top() -> Json<Value> {
    let client = reqwest::Client::new();
    match client.get("https://www.broadcastify.com/calls/status/topFeed")
        .send().await
    {
        Ok(resp) if resp.status().is_success() => {
            if let Ok(data) = resp.json::<Value>().await {
                return Json(data);
            }
        }
        _ => {}
    }
    Json(json!([]))
}

pub async fn radio_openmhz_systems() -> Json<Value> {
    let client = reqwest::Client::new();
    match client.get("https://api.openmhz.com/systems")
        .send().await
    {
        Ok(resp) if resp.status().is_success() => {
            if let Ok(data) = resp.json::<Value>().await {
                return Json(data);
            }
        }
        _ => {}
    }
    Json(json!([]))
}

pub async fn radio_openmhz_calls(Path(sys_name): Path<String>) -> Json<Value> {
    let client = reqwest::Client::new();
    match client.get(format!("https://api.openmhz.com/{}/calls", sys_name))
        .send().await
    {
        Ok(resp) if resp.status().is_success() => {
            if let Ok(data) = resp.json::<Value>().await {
                return Json(data);
            }
        }
        _ => {}
    }
    Json(json!([]))
}

pub async fn radio_nearest(Query(params): Query<LatLngQuery>) -> Json<Value> {
    Json(json!({"lat": params.lat, "lng": params.lng, "message": "nearest system lookup"}))
}

pub async fn get_api_keys(
    State(store): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if !check_admin(&store, &headers) {
        return (StatusCode::UNAUTHORIZED, Json(json!({"error": "Unauthorized"}))).into_response();
    }
    Json(json!({
        "ais_api_key": store.config.ais_api_key.as_ref().map(|_| "***configured***"),
        "opensky_client_id": store.config.opensky_client_id.as_ref().map(|_| "***configured***"),
    })).into_response()
}

pub async fn update_api_keys(
    State(store): State<AppState>,
    headers: HeaderMap,
    Json(_body): Json<Value>,
) -> impl IntoResponse {
    if !check_admin(&store, &headers) {
        return StatusCode::UNAUTHORIZED;
    }
    StatusCode::OK
}

pub async fn get_news_feeds() -> Json<Value> {
    Json(json!({
        "feeds": [
            {"name": "NPR World", "url": "https://feeds.npr.org/1004/rss.xml", "weight": 3},
            {"name": "BBC World", "url": "https://feeds.bbci.co.uk/news/world/rss.xml", "weight": 5},
            {"name": "Al Jazeera", "url": "https://www.aljazeera.com/xml/rss/all.xml", "weight": 4},
        ]
    }))
}

pub async fn update_news_feeds(
    State(store): State<AppState>,
    headers: HeaderMap,
    Json(_body): Json<Value>,
) -> impl IntoResponse {
    if !check_admin(&store, &headers) {
        return StatusCode::UNAUTHORIZED;
    }
    StatusCode::OK
}

pub async fn reset_news_feeds() -> StatusCode {
    StatusCode::OK
}

pub async fn force_refresh(State(_store): State<AppState>) -> StatusCode {
    // Trigger immediate re-fetch (in a real impl, would signal fetchers)
    StatusCode::OK
}

pub async fn debug_latest(State(store): State<AppState>) -> Json<Value> {
    let keys = store.all_keys();
    let mut info = json!({});
    for key in &keys {
        let count = store.get(key)
            .and_then(|v| v.as_array().map(|a| a.len()))
            .unwrap_or(0);
        info[key] = json!(count);
    }
    Json(info)
}

pub async fn ais_feed(
    State(store): State<AppState>,
    Json(body): Json<Value>,
) -> StatusCode {
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
        let lat = item
            .get("lat")
            .or_else(|| item.get("latitude"))
            .and_then(|v| v.as_f64().or_else(|| v.as_str()?.parse().ok()));
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

    let url = "https://planetarycomputer.microsoft.com/api/stac/v1/search";

    let body = json!({
        "collections": ["sentinel-2-l2a"],
        "intersects": {
            "type": "Point",
            "coordinates": [params.lng, params.lat],
        },
        "limit": 1,
        "sortby": [{"field": "datetime", "direction": "desc"}],
        "query": {
            "eo:cloud_cover": {"lt": 30}
        }
    });

    match client.post(url).json(&body).send().await {
        Ok(resp) if resp.status().is_success() => {
            if let Ok(data) = resp.json::<Value>().await {
                return Json(data);
            }
        }
        _ => {}
    }
    Json(json!({"features": []}))
}

fn check_admin(store: &AppState, headers: &HeaderMap) -> bool {
    match &store.config.admin_key {
        Some(key) => {
            headers.get("x-admin-key")
                .and_then(|v| v.to_str().ok())
                .map(|v| v == key)
                .unwrap_or(false)
        }
        None => true, // No admin key = open access
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
