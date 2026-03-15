use std::sync::Arc;
use crate::store::DataStore;
use tokio::time::{interval, Duration};

mod flights;
mod ships;
mod satellites;
mod earthquakes;
mod news;
mod fires;
mod gdelt;
mod infrastructure;
mod financial;
mod space_weather;
mod weather;
pub mod retry;
pub mod plane_alert;
pub mod yacht_alert;
pub mod military;
pub mod carrier_tracker;
pub mod liveuamap;
pub mod geopolitics;
pub mod sentinel;
pub mod cctv;
pub mod kiwisdr;
pub mod radio_intercept;
pub mod region_dossier;

pub fn spawn_all(store: Arc<DataStore>) {
    // Fast tier - 60s
    spawn_periodic("flights", store.clone(), Duration::from_secs(60), |s| async move {
        if let Err(e) = flights::fetch(&s).await { tracing::warn!("flights: {}", e); }
    });
    spawn_periodic("ships", store.clone(), Duration::from_secs(60), |s| async move {
        if let Err(e) = ships::fetch(&s).await { tracing::warn!("ships: {}", e); }
    });
    spawn_periodic("satellites", store.clone(), Duration::from_secs(60), |s| async move {
        if let Err(e) = satellites::fetch(&s).await { tracing::warn!("satellites: {}", e); }
    });

    // Medium tier - 120s
    spawn_periodic("earthquakes", store.clone(), Duration::from_secs(120), |s| async move {
        if let Err(e) = earthquakes::fetch(&s).await { tracing::warn!("earthquakes: {}", e); }
    });
    spawn_periodic("fires", store.clone(), Duration::from_secs(120), |s| async move {
        if let Err(e) = fires::fetch(&s).await { tracing::warn!("fires: {}", e); }
    });
    spawn_periodic("space_weather", store.clone(), Duration::from_secs(120), |s| async move {
        if let Err(e) = space_weather::fetch(&s).await { tracing::warn!("space_weather: {}", e); }
    });

    // Slow tier - 300s
    spawn_periodic("news", store.clone(), Duration::from_secs(300), |s| async move {
        if let Err(e) = news::fetch(&s).await { tracing::warn!("news: {}", e); }
    });
    spawn_periodic("financial", store.clone(), Duration::from_secs(300), |s| async move {
        if let Err(e) = financial::fetch(&s).await { tracing::warn!("financial: {}", e); }
    });
    spawn_periodic("infrastructure", store.clone(), Duration::from_secs(300), |s| async move {
        if let Err(e) = infrastructure::fetch(&s).await { tracing::warn!("infrastructure: {}", e); }
    });
    spawn_periodic("weather", store.clone(), Duration::from_secs(120), |s| async move {
        if let Err(e) = weather::fetch(&s).await { tracing::warn!("weather: {}", e); }
    });

    // Very slow tier - 900s
    spawn_periodic("gdelt", store.clone(), Duration::from_secs(900), |s| async move {
        if let Err(e) = gdelt::fetch(&s).await { tracing::warn!("gdelt: {}", e); }
    });

    tracing::info!("All background fetchers spawned");
}

fn spawn_periodic<F, Fut>(name: &'static str, store: Arc<DataStore>, period: Duration, f: F)
where
    F: Fn(Arc<DataStore>) -> Fut + Send + 'static,
    Fut: std::future::Future<Output = ()> + Send,
{
    tokio::spawn(async move {
        // Initial fetch immediately
        tracing::info!("Fetcher '{}' starting initial fetch", name);
        f(store.clone()).await;

        let mut ticker = interval(period);
        ticker.tick().await; // skip first immediate tick
        loop {
            ticker.tick().await;
            tracing::debug!("Fetcher '{}' running", name);
            f(store.clone()).await;
        }
    });
}
