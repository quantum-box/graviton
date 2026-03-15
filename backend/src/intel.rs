use chrono::{DateTime, Utc};
use serde_json::{json, Value};
use sqlx::Row;

use crate::store::{AlertEvent, DataStore};

pub async fn compute_sensor_fusion(store: &DataStore) -> Value {
    let flights = collect_items(
        &[store.get("commercial_flights"), store.get("private_flights"), store.get("private_jets"), store.get("military_flights"), store.get("tracked_flights"), store.get("uavs")],
        "aircraft",
    );
    let ships = collect_items(&[store.get("ships")], "ship");
    let fused = flights
        .into_iter()
        .chain(ships)
        .map(|item| {
            let source = item.get("source").and_then(Value::as_str).unwrap_or("synthetic");
            json!({
                "id": item.get("icao24").or_else(|| item.get("mmsi")).cloned().unwrap_or_else(|| json!("unknown")),
                "kind": item.get("kind").cloned().unwrap_or_else(|| json!("track")),
                "lat": item.get("lat").cloned().unwrap_or_else(|| json!(0.0)),
                "lng": item.get("lng").cloned().unwrap_or_else(|| json!(0.0)),
                "label": item.get("callsign").or_else(|| item.get("name")).cloned().unwrap_or(Value::Null),
                "sources": [source],
                "confidence": if source.contains("alert") { 0.92 } else { 0.74 },
            })
        })
        .collect::<Vec<_>>();
    json!(fused)
}

fn collect_items(values: &[Option<Value>], kind: &str) -> Vec<Value> {
    values
        .iter()
        .filter_map(|v| v.as_ref())
        .filter_map(Value::as_array)
        .flat_map(|items| items.iter().cloned())
        .map(|mut item| {
            item["kind"] = json!(kind);
            item
        })
        .collect()
}

pub async fn trajectories(store: &DataStore, object_type: &str, object_id: &str) -> anyhow::Result<Value> {
    let rows = sqlx::query(
        "SELECT ts, lat, lng, speed, heading FROM object_history WHERE object_type = $1 AND object_id = $2 ORDER BY ts DESC LIMIT 100",
    )
    .bind(object_type)
    .bind(object_id)
    .fetch_all(&store.db)
    .await?;
    let points = rows
        .into_iter()
        .rev()
        .map(|row| {
            json!({
                "ts": row.get::<DateTime<Utc>, _>("ts").to_rfc3339(),
                "lat": row.get::<f64, _>("lat"),
                "lng": row.get::<f64, _>("lng"),
                "speed": row.try_get::<f64, _>("speed").ok(),
                "heading": row.try_get::<f64, _>("heading").ok(),
            })
        })
        .collect::<Vec<_>>();
    Ok(json!({ "object_id": object_id, "object_type": object_type, "points": points }))
}

pub async fn prediction(store: &DataStore, object_type: &str, object_id: &str) -> anyhow::Result<Value> {
    let route = trajectories(store, object_type, object_id).await?;
    let points = route.get("points").and_then(Value::as_array).cloned().unwrap_or_default();
    if points.len() < 2 {
        return Ok(json!({ "object_id": object_id, "object_type": object_type, "line": [] }));
    }
    let a = &points[points.len() - 2];
    let b = &points[points.len() - 1];
    let lat1 = a.get("lat").and_then(Value::as_f64).unwrap_or(0.0);
    let lng1 = a.get("lng").and_then(Value::as_f64).unwrap_or(0.0);
    let lat2 = b.get("lat").and_then(Value::as_f64).unwrap_or(0.0);
    let lng2 = b.get("lng").and_then(Value::as_f64).unwrap_or(0.0);
    let dlat = lat2 - lat1;
    let dlng = lng2 - lng1;
    Ok(json!({
        "object_id": object_id,
        "object_type": object_type,
        "line": [
            {"lat": lat2, "lng": lng2},
            {"lat": lat2 + dlat, "lng": lng2 + dlng},
            {"lat": lat2 + dlat * 2.0, "lng": lng2 + dlng * 2.0}
        ]
    }))
}

pub async fn anomalies(store: &DataStore) -> anyhow::Result<Value> {
    let rows = sqlx::query(
        r#"
        SELECT object_type, object_id, AVG(speed) AS avg_speed, MAX(speed) AS max_speed, COUNT(*) AS samples
        FROM object_history
        WHERE ts > NOW() - INTERVAL '6 hours'
        GROUP BY object_type, object_id
        HAVING MAX(speed) > COALESCE(AVG(speed), 0) * 1.8 OR MAX(speed) > 600
        ORDER BY MAX(speed) DESC
        LIMIT 50
        "#,
    )
    .fetch_all(&store.db)
    .await?;
    Ok(json!(rows
        .into_iter()
        .map(|row| {
            json!({
                "object_type": row.get::<String, _>("object_type"),
                "object_id": row.get::<String, _>("object_id"),
                "avg_speed": row.try_get::<f64, _>("avg_speed").ok(),
                "max_speed": row.try_get::<f64, _>("max_speed").ok(),
                "samples": row.get::<i64, _>("samples"),
                "anomaly": "speed_spike",
            })
        })
        .collect::<Vec<_>>()))
}

pub async fn correlations(store: &DataStore) -> anyhow::Result<Value> {
    let military = store.get("military_flights").unwrap_or(json!([]));
    let news = store.get("news").unwrap_or(json!([]));
    let news_items = news.as_array().cloned().unwrap_or_default();
    let items = military
        .as_array()
        .cloned()
        .unwrap_or_default()
        .iter()
        .take(20)
        .filter_map(|flight| {
            let lat = flight.get("lat").and_then(Value::as_f64)?;
            let lng = flight.get("lng").and_then(Value::as_f64)?;
            let match_news = news_items
                .iter()
                .find(|entry| {
                    let nlat = entry.get("lat").and_then(Value::as_f64).unwrap_or(999.0);
                    let nlng = entry.get("lng").and_then(Value::as_f64).unwrap_or(999.0);
                    (lat - nlat).abs() < 2.5 && (lng - nlng).abs() < 2.5
                })?;
            Some(json!({
                "kind": "military_plus_conflict_signal",
                "flight": flight.get("callsign").cloned().unwrap_or_else(|| json!("unknown")),
                "news_title": match_news.get("title").cloned().unwrap_or_else(|| json!("unknown")),
                "lat": lat,
                "lng": lng,
            }))
        })
        .collect::<Vec<_>>();
    Ok(json!(items))
}

pub async fn report(store: &DataStore) -> anyhow::Result<String> {
    let anomalies = anomalies(store).await?;
    let correlations = correlations(store).await?;
    let alerts = store.latest_alerts.read().clone();
    Ok(format!(
        "# Graviton Situation Report\n\n- Generated: {}\n- Active alerts: {}\n- Speed anomalies: {}\n- Correlated military/conflict signals: {}\n\n## Recent Alerts\n{}\n",
        Utc::now().to_rfc3339(),
        alerts.len(),
        anomalies.as_array().map(|v| v.len()).unwrap_or(0),
        correlations.as_array().map(|v| v.len()).unwrap_or(0),
        alerts
            .iter()
            .take(10)
            .map(|a| format!("- [{}] {}: {}", a.severity, a.title, a.message))
            .collect::<Vec<_>>()
            .join("\n")
    ))
}

pub async fn detect_rule_alerts(store: &DataStore) -> anyhow::Result<()> {
    let military = store.get("military_flights").unwrap_or(json!([]));
    for flight in military.as_array().unwrap_or(&Vec::new()).iter().take(20) {
        let lat = flight.get("lat").and_then(Value::as_f64).unwrap_or(0.0);
        let lng = flight.get("lng").and_then(Value::as_f64).unwrap_or(0.0);
        let in_sensitive_zone = (20.0..=35.0).contains(&lat) && (120.0..=145.0).contains(&lng);
        if in_sensitive_zone {
            store
                .emit_alert(AlertEvent {
                    id: format!("airspace-{}", flight.get("icao24").and_then(Value::as_str).unwrap_or("unknown")),
                    severity: "high".to_string(),
                    title: "Sensitive airspace ingress".to_string(),
                    message: format!(
                        "{} entered monitored airspace",
                        flight.get("callsign").and_then(Value::as_str).unwrap_or("Unknown flight")
                    ),
                    source: "rule_engine".to_string(),
                    entity_id: flight.get("icao24").and_then(Value::as_str).map(ToString::to_string),
                    entity_type: Some("aircraft".to_string()),
                    lat: Some(lat),
                    lng: Some(lng),
                    created_at: Utc::now().to_rfc3339(),
                })
                .await;
        }
    }
    let anomalies = anomalies(store).await?;
    for item in anomalies.as_array().unwrap_or(&Vec::new()).iter().take(10) {
        store
            .emit_alert(AlertEvent {
                id: format!(
                    "anomaly-{}-{}",
                    item.get("object_type").and_then(Value::as_str).unwrap_or("track"),
                    item.get("object_id").and_then(Value::as_str).unwrap_or("unknown")
                ),
                severity: "medium".to_string(),
                title: "Anomalous movement".to_string(),
                message: format!(
                    "{}:{} exceeded its learned speed envelope",
                    item.get("object_type").and_then(Value::as_str).unwrap_or("track"),
                    item.get("object_id").and_then(Value::as_str).unwrap_or("unknown")
                ),
                source: "anomaly_detector".to_string(),
                entity_id: item.get("object_id").and_then(Value::as_str).map(ToString::to_string),
                entity_type: item.get("object_type").and_then(Value::as_str).map(ToString::to_string),
                lat: None,
                lng: None,
                created_at: Utc::now().to_rfc3339(),
            })
            .await;
    }
    Ok(())
}
