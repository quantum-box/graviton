use anyhow::{anyhow, Result};
use parking_lot::Mutex;
use rand::Rng;
use std::{
    collections::HashMap,
    future::Future,
    sync::{Arc, OnceLock},
    time::{Duration, Instant},
};
use tokio::time::sleep;

#[derive(Debug, Clone)]
struct CircuitState {
    failures: u32,
    opened_until: Option<Instant>,
}

static CIRCUITS: OnceLock<Arc<Mutex<HashMap<String, CircuitState>>>> = OnceLock::new();

fn circuits() -> &'static Arc<Mutex<HashMap<String, CircuitState>>> {
    CIRCUITS.get_or_init(|| Arc::new(Mutex::new(HashMap::new())))
}

pub fn circuit_open(name: &str) -> bool {
    let mut guard = circuits().lock();
    let Some(state) = guard.get_mut(name) else {
        return false;
    };
    if let Some(until) = state.opened_until {
        if Instant::now() < until {
            return true;
        }
        state.opened_until = None;
    }
    false
}

pub async fn with_retry_async<F, Fut, T>(name: &str, max_retries: usize, f: F) -> Result<T>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T>>,
{
    if circuit_open(name) {
        return Err(anyhow!("circuit breaker open for {}", name));
    }

    let mut last_err = None;
    for attempt in 0..=max_retries {
        match f().await {
            Ok(value) => {
                let mut guard = circuits().lock();
                guard.insert(
                    name.to_string(),
                    CircuitState {
                        failures: 0,
                        opened_until: None,
                    },
                );
                return Ok(value);
            }
            Err(err) => {
                last_err = Some(err);
                let delay_secs = (2_u64.pow(attempt as u32)).min(20);
                if attempt < max_retries {
                    let jitter = rand::thread_rng().gen_range(0..=400);
                    tracing::warn!(
                        "{} failed attempt {}/{}; retrying in {}ms",
                        name,
                        attempt + 1,
                        max_retries + 1,
                        delay_secs * 1000 + jitter
                    );
                    sleep(Duration::from_millis(delay_secs * 1000 + jitter)).await;
                }
            }
        }
    }

    let mut guard = circuits().lock();
    let state = guard.entry(name.to_string()).or_insert(CircuitState {
        failures: 0,
        opened_until: None,
    });
    state.failures += 1;
    if state.failures >= 3 {
        state.opened_until = Some(Instant::now() + Duration::from_secs(120));
    }
    Err(last_err.unwrap_or_else(|| anyhow!("retry wrapper failed without error")))
}

