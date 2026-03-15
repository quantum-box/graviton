use crate::store::DataStore;
use serde_json::{json, Value};
use std::sync::Arc;

use super::{carrier_tracker, yacht_alert};

pub async fn fetch(store: &Arc<DataStore>) -> anyhow::Result<()> {
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
    Ok(())
}
