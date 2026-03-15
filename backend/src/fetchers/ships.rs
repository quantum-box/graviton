use crate::store::DataStore;
use serde_json::{json, Value};
use std::sync::Arc;

use super::{carrier_tracker, yacht_alert};

pub async fn fetch(store: &Arc<DataStore>) -> anyhow::Result<()> {
    if let Some(cached) = store.cache_get_json("fetcher:ships").await.ok().flatten() {
        if let Some(items) = cached.as_array() {
            store.set("ships", json!(items));
        }
    }
    let mut ships = carrier_tracker::carrier_positions();
    ships.extend(
        store
            .get("ais_feed_snapshot")
            .and_then(|v| v.as_array().cloned())
            .unwrap_or_default(),
    );
    let ships: Vec<Value> = ships.into_iter().map(yacht_alert::enrich_ship).collect();
    store.set("ships", json!(ships));
    store.update_etag("fast");
    let _ = store.cache_set_json("fetcher:ships", &json!(store.get("ships").unwrap_or(json!([]))), 120).await;
    let _ = store.persist_positions("ship", "polling", &store.get("ships").and_then(|v| v.as_array().cloned()).unwrap_or_default()).await;
    let fused = crate::intel::compute_sensor_fusion(store).await;
    store.set("fused_objects", fused);
    Ok(())
}
