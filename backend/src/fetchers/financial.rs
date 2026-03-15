use crate::store::DataStore;
use serde_json::{json, Value};
use std::sync::Arc;

const DEFENSE_STOCKS: &[(&str, &str)] = &[
    ("LMT", "Lockheed Martin"),
    ("RTX", "RTX Corp"),
    ("NOC", "Northrop Grumman"),
    ("BA", "Boeing"),
    ("GD", "General Dynamics"),
    ("LHX", "L3Harris"),
];

const OIL_SYMBOLS: &[(&str, &str)] = &[
    ("CL=F", "WTI Crude"),
    ("BZ=F", "Brent Crude"),
];

pub async fn fetch(store: &Arc<DataStore>) -> anyhow::Result<()> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()?;

    let mut stocks: Vec<Value> = Vec::new();
    let mut oil: Vec<Value> = Vec::new();

    // Fetch stock data from Yahoo Finance v7 API
    let all_symbols: Vec<&str> = DEFENSE_STOCKS
        .iter()
        .map(|(s, _)| *s)
        .chain(OIL_SYMBOLS.iter().map(|(s, _)| *s))
        .collect();

    let symbols_str = all_symbols.join(",");
    let url = format!(
        "https://query1.finance.yahoo.com/v7/finance/quote?symbols={}",
        symbols_str
    );

    match client
        .get(&url)
        .header("User-Agent", "Mozilla/5.0")
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => {
            if let Ok(data) = resp.json::<Value>().await {
                if let Some(results) = data
                    .get("quoteResponse")
                    .and_then(|q| q.get("result"))
                    .and_then(|r| r.as_array())
                {
                    for quote in results {
                        let symbol = quote
                            .get("symbol")
                            .and_then(|v| v.as_str())
                            .unwrap_or("");
                        let price = quote
                            .get("regularMarketPrice")
                            .and_then(|v| v.as_f64())
                            .unwrap_or(0.0);
                        let change = quote
                            .get("regularMarketChangePercent")
                            .and_then(|v| v.as_f64())
                            .unwrap_or(0.0);

                        if DEFENSE_STOCKS.iter().any(|(s, _)| *s == symbol) {
                            let name = DEFENSE_STOCKS
                                .iter()
                                .find(|(s, _)| *s == symbol)
                                .map(|(_, n)| *n)
                                .unwrap_or(symbol);
                            stocks.push(json!({
                                "symbol": symbol,
                                "name": name,
                                "price": price,
                                "change_pct": (change * 100.0).round() / 100.0,
                            }));
                        } else if OIL_SYMBOLS.iter().any(|(s, _)| *s == symbol) {
                            let name = OIL_SYMBOLS
                                .iter()
                                .find(|(s, _)| *s == symbol)
                                .map(|(_, n)| *n)
                                .unwrap_or(symbol);
                            oil.push(json!({
                                "symbol": symbol,
                                "name": name,
                                "price": price,
                                "change_pct": (change * 100.0).round() / 100.0,
                            }));
                        }
                    }
                }
            }
        }
        _ => tracing::warn!("Yahoo Finance fetch failed"),
    }

    store.set("stocks", json!(stocks));
    store.set("oil", json!(oil));
    store.update_etag("slow");
    tracing::info!("Financial: {} stocks, {} oil", stocks.len(), oil.len());
    Ok(())
}
