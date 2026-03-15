use anyhow::Result;
use serde_json::{json, Value};

pub async fn fetch(client: &reqwest::Client) -> Result<Vec<Value>> {
    let resp = client.get("http://rx.linkfanel.net/kiwisdr_com.js").send().await?;
    if !resp.status().is_success() {
        return Ok(Vec::new());
    }
    let body = resp.text().await?;
    let Some(start) = body.find('[') else { return Ok(Vec::new()) };
    let Some(end) = body.rfind(']') else { return Ok(Vec::new()) };
    let arr = serde_json::from_str::<Vec<Value>>(&body[start..=end]).unwrap_or_default();
    Ok(arr
        .into_iter()
        .filter_map(|item| {
            let gps = item.get("gps")?.as_str()?;
            let parts: Vec<_> = gps.split(',').collect();
            let lat = parts.first()?.trim().parse::<f64>().ok()?;
            let lng = parts.get(1)?.trim().parse::<f64>().ok()?;
            Some(json!({
                "name": item.get("name"),
                "lat": lat,
                "lng": lng,
                "url": item.get("url"),
                "antenna": item.get("antenna"),
                "bands": item.get("bands"),
                "users": item.get("users"),
                "users_max": item.get("users_max"),
            }))
        })
        .collect())
}

