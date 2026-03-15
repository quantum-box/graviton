use axum::{
    extract::{
        Path, Query, State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse},
    Json,
};
use chrono::Utc;
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::Row;
use uuid::Uuid;

use crate::{auth, intel, AppState};

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

#[derive(Deserialize)]
pub struct TrackQuery {
    pub object_type: String,
    pub object_id: String,
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
    let data = store.fast_snapshot();
    let mut resp_headers = HeaderMap::new();
    resp_headers.insert("etag", format!("\"{}\"", etag).parse().unwrap());
    resp_headers.insert("cache-control", "no-cache".parse().unwrap());
    (StatusCode::OK, resp_headers, serde_json::to_string(&data).unwrap()).into_response()
}

pub async fn live_data_slow(State(store): State<AppState>, headers: HeaderMap) -> impl IntoResponse {
    let etag = store.get_etag("slow");
    if let Some(if_none_match) = headers.get("if-none-match") {
        if let Ok(val) = if_none_match.to_str() {
            if val.trim_matches('"') == etag {
                return (StatusCode::NOT_MODIFIED, HeaderMap::new(), String::new()).into_response();
            }
        }
    }
    let data = store.slow_snapshot();
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
        "alerts": store.latest_alerts.read().len(),
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

pub async fn update_viewport(State(store): State<AppState>, Json(viewport): Json<crate::store::Viewport>) -> StatusCode {
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

pub async fn update_api_keys(State(store): State<AppState>, headers: HeaderMap, Json(body): Json<Value>) -> impl IntoResponse {
    if !check_admin(&store, &headers) {
        return StatusCode::UNAUTHORIZED;
    }
    let values = body
        .as_object()
        .map(|obj| {
            obj.iter()
                .filter_map(|(k, v)| v.as_str().map(|s| (k.to_string(), s.to_string())))
                .collect::<std::collections::HashMap<_, _>>()
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

pub async fn update_news_feeds(State(store): State<AppState>, headers: HeaderMap, Json(body): Json<Value>) -> impl IntoResponse {
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
        StatusCode::BAD_REQUEST
    }
}

pub async fn force_refresh(State(store): State<AppState>) -> Json<Value> {
    crate::fetchers::spawn_manual_refresh(store.clone());
    Json(json!({ "status": "refresh scheduled" }))
}

pub async fn debug_latest(State(store): State<AppState>) -> Json<Value> {
    Json(json!({
        "fast": store.fast_snapshot(),
        "slow": store.slow_snapshot(),
    }))
}

pub async fn ais_feed(State(store): State<AppState>, Json(body): Json<Value>) -> StatusCode {
    store.set("ais_feed_snapshot", body);
    StatusCode::OK
}

pub async fn sentinel_search(Query(params): Query<LatLngQuery>) -> Json<Value> {
    let client = reqwest::Client::new();
    let query = crate::fetchers::sentinel::search(&client, params.lat, params.lng).await.unwrap_or_else(|_| json!([]));
    Json(json!({
        "results": query,
        "tile_url": "https://tiles.maps.eox.at/wmts/1.0.0/s2cloudless-2024_3857/default/g/{z}/{y}/{x}.jpg"
    }))
}

pub async fn ws_live(ws: WebSocketUpgrade, State(store): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, store))
}

async fn handle_socket(mut socket: WebSocket, store: AppState) {
    let _ = socket
        .send(Message::Text(
            json!({
                "type": "snapshot",
                "fast": store.fast_snapshot(),
                "slow": store.slow_snapshot(),
            })
            .to_string()
            .into(),
        ))
        .await;
    let mut rx = store.subscribe();
    loop {
        tokio::select! {
            recv = rx.recv() => {
                match recv {
                    Ok(event) => {
                        if socket.send(Message::Text(serde_json::to_string(&event).unwrap_or_else(|_| "{}".to_string()).into())).await.is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
            incoming = socket.recv() => {
                match incoming {
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Ok(_)) => {}
                    Some(Err(_)) => break,
                }
            }
        }
    }
}

pub async fn auth_register(State(store): State<AppState>, Json(payload): Json<auth::AuthPayload>) -> impl IntoResponse {
    match auth::register(&store, payload).await {
        Ok(resp) => (StatusCode::CREATED, Json(json!(resp))).into_response(),
        Err(err) => (StatusCode::BAD_REQUEST, Json(json!({ "error": err.to_string() }))).into_response(),
    }
}

pub async fn auth_login(State(store): State<AppState>, Json(payload): Json<auth::AuthPayload>) -> impl IntoResponse {
    match auth::login(&store, payload).await {
        Ok(resp) => (StatusCode::OK, Json(json!(resp))).into_response(),
        Err(_) => (StatusCode::UNAUTHORIZED, Json(json!({ "error": "Invalid credentials" }))).into_response(),
    }
}

pub async fn auth_me(State(store): State<AppState>, headers: HeaderMap) -> impl IntoResponse {
    if let Some(claims) = auth::auth_from_headers(&store, &headers) {
        (StatusCode::OK, Json(json!({ "user": claims }))).into_response()
    } else {
        (StatusCode::UNAUTHORIZED, Json(json!({ "error": "Unauthorized" }))).into_response()
    }
}

pub async fn analysis_trajectories(State(store): State<AppState>, Query(query): Query<TrackQuery>) -> impl IntoResponse {
    match intel::trajectories(&store, &query.object_type, &query.object_id).await {
        Ok(value) => Json(value).into_response(),
        Err(err) => (StatusCode::BAD_REQUEST, Json(json!({ "error": err.to_string() }))).into_response(),
    }
}

pub async fn analysis_predictions(State(store): State<AppState>, Query(query): Query<TrackQuery>) -> impl IntoResponse {
    match intel::prediction(&store, &query.object_type, &query.object_id).await {
        Ok(value) => Json(value).into_response(),
        Err(err) => (StatusCode::BAD_REQUEST, Json(json!({ "error": err.to_string() }))).into_response(),
    }
}

pub async fn analysis_anomalies(State(store): State<AppState>) -> impl IntoResponse {
    match intel::anomalies(&store).await {
        Ok(value) => Json(value).into_response(),
        Err(err) => (StatusCode::BAD_REQUEST, Json(json!({ "error": err.to_string() }))).into_response(),
    }
}

pub async fn analysis_correlations(State(store): State<AppState>) -> impl IntoResponse {
    match intel::correlations(&store).await {
        Ok(value) => Json(value).into_response(),
        Err(err) => (StatusCode::BAD_REQUEST, Json(json!({ "error": err.to_string() }))).into_response(),
    }
}

pub async fn report_summary(State(store): State<AppState>) -> impl IntoResponse {
    match intel::report(&store).await {
        Ok(markdown) => (StatusCode::OK, [("content-type", "text/markdown; charset=utf-8")], markdown).into_response(),
        Err(err) => (StatusCode::BAD_REQUEST, Json(json!({ "error": err.to_string() }))).into_response(),
    }
}

pub async fn fusion_objects(State(store): State<AppState>) -> Json<Value> {
    let fused = intel::compute_sensor_fusion(&store).await;
    store.set("fused_objects", fused.clone());
    Json(fused)
}

pub async fn shared_list(State(store): State<AppState>, headers: HeaderMap) -> impl IntoResponse {
    if auth::auth_from_headers(&store, &headers).is_none() {
        return (StatusCode::UNAUTHORIZED, Json(json!({ "error": "Unauthorized" }))).into_response();
    }
    let rows = sqlx::query("SELECT id, kind, title, payload, created_at FROM shared_objects ORDER BY created_at DESC LIMIT 200")
        .fetch_all(&store.db)
        .await
        .unwrap_or_default();
    Json(json!(rows
        .into_iter()
        .map(|row| {
            json!({
                "id": row.get::<Uuid, _>("id").to_string(),
                "kind": row.get::<String, _>("kind"),
                "title": row.get::<String, _>("title"),
                "payload": row.get::<serde_json::Value, _>("payload"),
                "created_at": row.get::<chrono::DateTime<Utc>, _>("created_at").to_rfc3339(),
            })
        })
        .collect::<Vec<_>>()))
    .into_response()
}

pub async fn shared_create(State(store): State<AppState>, headers: HeaderMap, Json(body): Json<Value>) -> impl IntoResponse {
    let Some(claims) = auth::auth_from_headers(&store, &headers) else {
        return (StatusCode::UNAUTHORIZED, Json(json!({ "error": "Unauthorized" }))).into_response();
    };
    let id = Uuid::new_v4();
    let title = body.get("title").and_then(Value::as_str).unwrap_or("Shared item");
    let kind = body.get("kind").and_then(Value::as_str).unwrap_or("marker");
    let payload = body.get("payload").cloned().unwrap_or_else(|| json!({}));
    let _ = sqlx::query("INSERT INTO shared_objects (id, user_id, kind, title, payload) VALUES ($1, $2, $3, $4, $5)")
        .bind(id)
        .bind(Uuid::parse_str(&claims.sub).ok())
        .bind(kind)
        .bind(title)
        .bind(payload.clone())
        .execute(&store.db)
        .await;
    (StatusCode::CREATED, Json(json!({ "id": id.to_string(), "title": title, "kind": kind, "payload": payload }))).into_response()
}

pub async fn webhook_get(State(store): State<AppState>, headers: HeaderMap) -> impl IntoResponse {
    let Some(claims) = auth::auth_from_headers(&store, &headers) else {
        return (StatusCode::UNAUTHORIZED, Json(json!({ "error": "Unauthorized" }))).into_response();
    };
    let row = sqlx::query("SELECT webhook_url FROM webhook_settings WHERE user_id = $1")
        .bind(Uuid::parse_str(&claims.sub).ok())
        .fetch_optional(&store.db)
        .await
        .ok()
        .flatten();
    Json(json!({ "webhook_url": row.and_then(|r| r.try_get::<String, _>("webhook_url").ok()) })).into_response()
}

pub async fn webhook_put(State(store): State<AppState>, headers: HeaderMap, Json(body): Json<Value>) -> impl IntoResponse {
    let Some(claims) = auth::auth_from_headers(&store, &headers) else {
        return (StatusCode::UNAUTHORIZED, Json(json!({ "error": "Unauthorized" }))).into_response();
    };
    let url = body.get("webhook_url").and_then(Value::as_str);
    let _ = sqlx::query(
        "INSERT INTO webhook_settings (user_id, webhook_url) VALUES ($1, $2) ON CONFLICT (user_id) DO UPDATE SET webhook_url = EXCLUDED.webhook_url",
    )
    .bind(Uuid::parse_str(&claims.sub).ok())
    .bind(url)
    .execute(&store.db)
    .await;
    StatusCode::OK.into_response()
}

pub async fn simulation_list(State(store): State<AppState>) -> Json<Value> {
    Json(store.get("simulation_markers").unwrap_or(json!([])))
}

pub async fn simulation_create(State(store): State<AppState>, Json(body): Json<Value>) -> impl IntoResponse {
    let mut items = store.get("simulation_markers").and_then(|v| v.as_array().cloned()).unwrap_or_default();
    let id = body.get("id").and_then(Value::as_str).unwrap_or("sim").to_string();
    let marker = json!({
        "id": if id == "sim" { format!("sim-{}", Uuid::new_v4()) } else { id },
        "name": body.get("name").and_then(Value::as_str).unwrap_or("Simulation marker"),
        "lat": body.get("lat").and_then(Value::as_f64).unwrap_or(35.0),
        "lng": body.get("lng").and_then(Value::as_f64).unwrap_or(135.0),
        "heading": body.get("heading").and_then(Value::as_f64).unwrap_or(90.0),
        "speed": body.get("speed").and_then(Value::as_f64).unwrap_or(0.12)
    });
    items.push(marker.clone());
    store.set("simulation_markers", json!(items));
    (StatusCode::CREATED, Json(marker)).into_response()
}

pub async fn simulation_update(State(store): State<AppState>, Path(id): Path<String>, Json(body): Json<Value>) -> impl IntoResponse {
    let mut items = store.get("simulation_markers").and_then(|v| v.as_array().cloned()).unwrap_or_default();
    for item in &mut items {
        if item.get("id").and_then(Value::as_str) == Some(id.as_str()) {
            if let Some(lat) = body.get("lat").and_then(Value::as_f64) {
                item["lat"] = json!(lat);
            }
            if let Some(lng) = body.get("lng").and_then(Value::as_f64) {
                item["lng"] = json!(lng);
            }
            if let Some(speed) = body.get("speed").and_then(Value::as_f64) {
                item["speed"] = json!(speed);
            }
        }
    }
    store.set("simulation_markers", json!(items.clone()));
    Json(json!(items)).into_response()
}

pub async fn c2_state(State(store): State<AppState>) -> impl IntoResponse {
    let watchlist = sqlx::query("SELECT id, target_id, target_type, note, created_at FROM watchlist ORDER BY created_at DESC LIMIT 100")
        .fetch_all(&store.db)
        .await
        .unwrap_or_default();
    let missions = sqlx::query("SELECT id, title, status, payload, created_at FROM missions ORDER BY created_at DESC LIMIT 100")
        .fetch_all(&store.db)
        .await
        .unwrap_or_default();
    Json(json!({
        "watchlist": watchlist.into_iter().map(|row| json!({
            "id": row.get::<Uuid, _>("id").to_string(),
            "target_id": row.get::<String, _>("target_id"),
            "target_type": row.get::<String, _>("target_type"),
            "note": row.try_get::<String, _>("note").ok(),
            "created_at": row.get::<chrono::DateTime<Utc>, _>("created_at").to_rfc3339(),
        })).collect::<Vec<_>>(),
        "missions": missions.into_iter().map(|row| json!({
            "id": row.get::<Uuid, _>("id").to_string(),
            "title": row.get::<String, _>("title"),
            "status": row.get::<String, _>("status"),
            "payload": row.get::<serde_json::Value, _>("payload"),
            "created_at": row.get::<chrono::DateTime<Utc>, _>("created_at").to_rfc3339(),
        })).collect::<Vec<_>>(),
        "alerts": store.latest_alerts.read().clone(),
    })).into_response()
}

pub async fn c2_watchlist_create(State(store): State<AppState>, headers: HeaderMap, Json(body): Json<Value>) -> impl IntoResponse {
    let Some(claims) = auth::auth_from_headers(&store, &headers) else {
        return (StatusCode::UNAUTHORIZED, Json(json!({ "error": "Unauthorized" }))).into_response();
    };
    let _ = sqlx::query("INSERT INTO watchlist (id, user_id, target_id, target_type, note) VALUES ($1, $2, $3, $4, $5)")
        .bind(Uuid::new_v4())
        .bind(Uuid::parse_str(&claims.sub).ok())
        .bind(body.get("target_id").and_then(Value::as_str).unwrap_or("unknown"))
        .bind(body.get("target_type").and_then(Value::as_str).unwrap_or("track"))
        .bind(body.get("note").and_then(Value::as_str))
        .execute(&store.db)
        .await;
    StatusCode::CREATED.into_response()
}

pub async fn c2_mission_create(State(store): State<AppState>, headers: HeaderMap, Json(body): Json<Value>) -> impl IntoResponse {
    let Some(claims) = auth::auth_from_headers(&store, &headers) else {
        return (StatusCode::UNAUTHORIZED, Json(json!({ "error": "Unauthorized" }))).into_response();
    };
    let title = body.get("title").and_then(Value::as_str).unwrap_or("Mission").to_string();
    let status = body.get("status").and_then(Value::as_str).unwrap_or("planned").to_string();
    let _ = sqlx::query("INSERT INTO missions (id, user_id, title, status, payload) VALUES ($1, $2, $3, $4, $5)")
        .bind(Uuid::new_v4())
        .bind(Uuid::parse_str(&claims.sub).ok())
        .bind(title)
        .bind(status)
        .bind(body)
        .execute(&store.db)
        .await;
    StatusCode::CREATED.into_response()
}

pub async fn c2_alert_rule_create(State(store): State<AppState>, headers: HeaderMap, Json(body): Json<Value>) -> impl IntoResponse {
    let Some(claims) = auth::auth_from_headers(&store, &headers) else {
        return (StatusCode::UNAUTHORIZED, Json(json!({ "error": "Unauthorized" }))).into_response();
    };
    let title = body.get("title").and_then(Value::as_str).unwrap_or("Alert rule").to_string();
    let _ = sqlx::query("INSERT INTO shared_objects (id, user_id, kind, title, payload) VALUES ($1, $2, $3, $4, $5)")
        .bind(Uuid::new_v4())
        .bind(Uuid::parse_str(&claims.sub).ok())
        .bind("alert_rule")
        .bind(title)
        .bind(body)
        .execute(&store.db)
        .await;
    StatusCode::CREATED.into_response()
}

pub async fn docs_html() -> Html<String> {
    Html(format!(
        r#"<!doctype html>
<html>
  <head>
    <meta charset="utf-8" />
    <title>Graviton API Docs</title>
    <link rel="stylesheet" href="https://unpkg.com/swagger-ui-dist@5/swagger-ui.css" />
  </head>
  <body>
    <div id="swagger-ui"></div>
    <script src="https://unpkg.com/swagger-ui-dist@5/swagger-ui-bundle.js"></script>
    <script>
      window.ui = SwaggerUIBundle({{ url: '/api/openapi.json', dom_id: '#swagger-ui' }});
    </script>
  </body>
</html>"#
    ))
}

pub async fn openapi_json() -> Json<Value> {
    Json(json!({
        "openapi": "3.0.3",
        "info": { "title": "Graviton API", "version": "0.2.0" },
        "paths": {
            "/api/live-data/fast": { "get": { "summary": "Fast tier live data" } },
            "/api/live-data/slow": { "get": { "summary": "Slow tier live data" } },
            "/api/ws/live": { "get": { "summary": "Realtime websocket stream" } },
            "/api/auth/register": { "post": { "summary": "Register local operator account" } },
            "/api/auth/login": { "post": { "summary": "Login and receive JWT" } },
            "/api/team/shared": { "get": { "summary": "List team shared objects" }, "post": { "summary": "Create shared object" } },
            "/api/analysis/trajectories": { "get": { "summary": "Object movement history" } },
            "/api/analysis/predictions": { "get": { "summary": "Linear next position estimate" } },
            "/api/analysis/anomalies": { "get": { "summary": "Current anomalies" } },
            "/api/analysis/correlations": { "get": { "summary": "Cross-source correlations" } },
            "/api/report/summary": { "get": { "summary": "Markdown report" } },
            "/api/fusion/objects": { "get": { "summary": "Sensor-fused tracks" } },
            "/api/simulation/markers": { "get": { "summary": "Simulation markers" }, "post": { "summary": "Create simulation marker" } },
            "/api/c2/state": { "get": { "summary": "Command panel state" } },
            "/api/docs": { "get": { "summary": "Swagger UI" } }
        }
    }))
}

fn check_admin(store: &AppState, headers: &HeaderMap) -> bool {
    let Some(expected) = store.config.admin_key.as_ref() else {
        return true;
    };
    headers
        .get("x-admin-key")
        .and_then(|v| v.to_str().ok())
        .map(|actual| actual == expected)
        .unwrap_or(false)
}
