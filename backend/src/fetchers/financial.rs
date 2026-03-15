use crate::store::DataStore;
use anyhow::Context;
use csv::StringRecord;
use serde_json::{json, Value};
use std::sync::Arc;

const DEFENSE_STOCKS: &[(&str, &str, &str)] = &[
    ("LMT", "lmt.us", "Lockheed Martin"),
    ("RTX", "rtx.us", "RTX Corp"),
    ("NOC", "noc.us", "Northrop Grumman"),
    ("BA", "ba.us", "Boeing"),
    ("GD", "gd.us", "General Dynamics"),
    ("LHX", "lhx.us", "L3Harris"),
];

const OIL_SYMBOLS: &[(&str, &str, &str)] = &[
    ("CL.F", "cl.f", "WTI Crude"),
    ("CB.F", "cb.f", "Brent Crude"),
];

pub async fn fetch(store: &Arc<DataStore>) -> anyhow::Result<()> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()?;

    let mut stocks: Vec<Value> = Vec::new();
    let mut oil: Vec<Value> = Vec::new();

    for (symbol, stooq_symbol, name) in DEFENSE_STOCKS {
        match fetch_stooq_quote(&client, stooq_symbol, symbol, name).await {
            Ok(item) => stocks.push(item),
            Err(err) => tracing::warn!("Stock fetch failed for {}: {}", symbol, err),
        }
    }

    for (symbol, stooq_symbol, name) in OIL_SYMBOLS {
        match fetch_stooq_quote(&client, stooq_symbol, symbol, name).await {
            Ok(item) => oil.push(item),
            Err(err) => tracing::warn!("Oil fetch failed for {}: {}", symbol, err),
        }
    }

    store.set("stocks", json!(stocks));
    store.set("oil", json!(oil));
    store.update_etag("slow");
    tracing::info!("Financial: {} stocks, {} oil", stocks.len(), oil.len());
    Ok(())
}

async fn fetch_stooq_quote(
    client: &reqwest::Client,
    stooq_symbol: &str,
    display_symbol: &str,
    display_name: &str,
) -> anyhow::Result<Value> {
    let url = format!("https://stooq.com/q/l/?s={stooq_symbol}&f=sd2t2ohlcvn&e=csv");
    let response = client
        .get(url)
        .header("User-Agent", "Graviton/1.0")
        .send()
        .await
        .context("request failed")?;
    let body = response.text().await.context("response body failed")?;

    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(body.as_bytes());
    let record = reader
        .records()
        .next()
        .transpose()
        .context("csv parse failed")?
        .context("empty csv response")?;

    parse_stooq_record(&record, display_symbol, display_name)
}

fn parse_stooq_record(record: &StringRecord, display_symbol: &str, display_name: &str) -> anyhow::Result<Value> {
    let close = parse_price(record.get(6)).context("missing close")?;
    let open = parse_price(record.get(3)).unwrap_or(close);
    let change_pct = if open.abs() > f64::EPSILON {
        ((close - open) / open) * 100.0
    } else {
        0.0
    };

    Ok(json!({
        "symbol": display_symbol,
        "name": record.get(8).filter(|value| *value != "N/D").unwrap_or(display_name),
        "price": (close * 100.0).round() / 100.0,
        "change_pct": (change_pct * 100.0).round() / 100.0,
        "date": record.get(1),
        "time": record.get(2),
    }))
}

fn parse_price(value: Option<&str>) -> Option<f64> {
    let raw = value?.trim();
    if raw.is_empty() || raw == "N/D" {
        return None;
    }
    raw.parse::<f64>().ok()
}
