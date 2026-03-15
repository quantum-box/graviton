use anyhow::Result;
use regex::Regex;
use serde_json::{json, Value};

pub async fn fetch_liveuamap(client: &reqwest::Client) -> Result<Vec<Value>> {
    let mut out = Vec::new();
    let resp = client
        .get("https://www.liveuamap.com")
        .header("User-Agent", "Graviton/1.0")
        .send()
        .await?;
    if !resp.status().is_success() {
        return Ok(out);
    }
    let html = resp.text().await?;
    let re = Regex::new(r#"data-lat="([^"]+)".*?data-lng="([^"]+)".*?data-title="([^"]+)""#)?;
    for cap in re.captures_iter(&html).take(150) {
        let lat = cap.get(1).and_then(|m| m.as_str().parse::<f64>().ok()).unwrap_or(0.0);
        let lng = cap.get(2).and_then(|m| m.as_str().parse::<f64>().ok()).unwrap_or(0.0);
        let title = html_escape(&cap[3]);
        if lat == 0.0 && lng == 0.0 {
            continue;
        }
        out.push(json!({
            "lat": lat,
            "lng": lng,
            "title": title,
            "region": "LiveUAmap",
            "link": "https://www.liveuamap.com",
        }));
    }
    Ok(out)
}

fn html_escape(input: &str) -> String {
    input
        .replace("&quot;", "\"")
        .replace("&amp;", "&")
        .replace("&#39;", "'")
}

