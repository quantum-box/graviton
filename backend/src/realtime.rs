use std::sync::Arc;

use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use tokio::time::{interval, Duration};
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::store::{DataStore, WsEvent};

pub fn spawn(store: Arc<DataStore>) {
    spawn_ais_stream(store.clone());
    spawn_aircraft_stream(store.clone());
    spawn_simulation(store);
}

fn spawn_ais_stream(store: Arc<DataStore>) {
    let url = store.config.ais_stream_url.clone();
    tokio::spawn(async move {
        if let Some(url) = url {
            if let Err(err) = run_ws_source(store.clone(), &url, "ship").await {
                tracing::warn!("AIS stream failed: {}", err);
            }
            return;
        }
        let mut ticker = interval(Duration::from_secs(3));
        loop {
            ticker.tick().await;
            let ships = store.get("ships").unwrap_or(json!([]));
            let payload = ships.as_array().cloned().unwrap_or_default().into_iter().take(8).collect::<Vec<_>>();
            let _ = store.broadcaster.send(WsEvent::Dataset {
                key: "ships_stream".to_string(),
                payload: json!(payload),
            });
        }
    });
}

fn spawn_aircraft_stream(store: Arc<DataStore>) {
    let url = store.config.aircraft_stream_url.clone();
    tokio::spawn(async move {
        if let Some(url) = url {
            if let Err(err) = run_ws_source(store.clone(), &url, "aircraft").await {
                tracing::warn!("Aircraft stream failed: {}", err);
            }
            return;
        }
        let mut ticker = interval(Duration::from_secs(3));
        loop {
            ticker.tick().await;
            let flights = store.get("military_flights").unwrap_or(json!([]));
            let payload = flights.as_array().cloned().unwrap_or_default().into_iter().take(12).collect::<Vec<_>>();
            let _ = store.broadcaster.send(WsEvent::Dataset {
                key: "aircraft_stream".to_string(),
                payload: json!(payload),
            });
        }
    });
}

fn spawn_simulation(store: Arc<DataStore>) {
    tokio::spawn(async move {
        let mut ticker = interval(Duration::from_secs(5));
        loop {
            ticker.tick().await;
            let mut markers = store
                .get("simulation_markers")
                .and_then(|v| v.as_array().cloned())
                .unwrap_or_default();
            if markers.is_empty() {
                markers.push(json!({
                    "id": "sim-alpha",
                    "name": "Sim Alpha",
                    "lat": 35.6762,
                    "lng": 139.6503,
                    "heading": 90.0,
                    "speed": 0.12,
                }));
            } else {
                for marker in &mut markers {
                    let lat = marker.get("lat").and_then(Value::as_f64).unwrap_or(0.0);
                    let lng = marker.get("lng").and_then(Value::as_f64).unwrap_or(0.0);
                    let speed = marker.get("speed").and_then(Value::as_f64).unwrap_or(0.12);
                    marker["lat"] = json!(lat + speed * 0.4);
                    marker["lng"] = json!(lng + speed);
                }
            }
            store.set("simulation_markers", json!(markers.clone()));
            let _ = store.broadcaster.send(WsEvent::Simulation {
                payload: json!(markers),
            });
        }
    });
}

async fn run_ws_source(store: Arc<DataStore>, url: &str, kind: &str) -> anyhow::Result<()> {
    let (mut stream, _) = connect_async(url).await?;
    let subscribe = match kind {
        "ship" => json!({
            "APIKey": store.config.ais_api_key.clone().unwrap_or_default(),
            "BoundingBoxes": [[[-90.0, -180.0], [90.0, 180.0]]]
        }),
        _ => json!({ "action": "subscribe", "topic": "aircraft" }),
    };
    stream.send(Message::Text(subscribe.to_string().into())).await?;
    while let Some(message) = stream.next().await {
        let message = message?;
        if let Message::Text(text) = message {
            let value: Value = serde_json::from_str(&text).unwrap_or_else(|_| json!({ "raw": text }));
            let _ = store.broadcaster.send(WsEvent::Dataset {
                key: format!("{}_stream", kind),
                payload: value,
            });
        }
    }
    Ok(())
}
