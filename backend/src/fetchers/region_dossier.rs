use anyhow::Result;
use serde_json::{json, Value};

pub async fn get_region_dossier(client: &reqwest::Client, lat: f64, lng: f64) -> Result<Value> {
    let geo = reverse_geocode(client, lat, lng).await?;
    if geo.get("country").and_then(Value::as_str).unwrap_or("").is_empty() {
        return Ok(json!({
            "coordinates": { "lat": lat, "lng": lng },
            "location": geo,
            "country": Value::Null,
            "local": Value::Null,
            "error": "No country data — possibly international waters or uninhabited area",
        }));
    }
    let country_code = geo.get("country_code").and_then(Value::as_str).unwrap_or("");
    let country_name = geo.get("country").and_then(Value::as_str).unwrap_or("");
    let city_name = geo.get("city").and_then(Value::as_str).unwrap_or("");
    let state_name = geo.get("state").and_then(Value::as_str).unwrap_or("");

    let country_data = fetch_country_data(client, country_code).await.unwrap_or_else(|_| json!({}));
    let leader_data = fetch_wikidata_leader(client, country_name).await.unwrap_or_else(|_| json!({}));
    let local_data = fetch_wiki_summary(client, if city_name.is_empty() { state_name } else { city_name }, country_name)
        .await
        .unwrap_or_else(|_| json!({}));

    let languages = country_data
        .get("languages")
        .and_then(Value::as_object)
        .map(|obj| obj.values().filter_map(Value::as_str).collect::<Vec<_>>())
        .unwrap_or_default();

    let currencies = country_data
        .get("currencies")
        .and_then(Value::as_object)
        .map(|obj| {
            obj.values()
                .filter_map(|v| {
                    let name = v.get("name").and_then(Value::as_str)?;
                    let symbol = v.get("symbol").and_then(Value::as_str).unwrap_or("");
                    Some(if symbol.is_empty() { name.to_string() } else { format!("{} ({})", name, symbol) })
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    Ok(json!({
        "coordinates": { "lat": lat, "lng": lng },
        "location": geo,
        "country": {
            "name": country_data.get("name").and_then(|v| v.get("common")).and_then(Value::as_str).unwrap_or(country_name),
            "official_name": country_data.get("name").and_then(|v| v.get("official")).and_then(Value::as_str).unwrap_or(""),
            "leader": leader_data.get("leader").and_then(Value::as_str).unwrap_or("Unknown"),
            "government_type": leader_data.get("government_type").and_then(Value::as_str).unwrap_or("Unknown"),
            "population": country_data.get("population").cloned().unwrap_or(json!(0)),
            "capital": country_data.get("capital").and_then(Value::as_array).and_then(|a| a.first()).and_then(Value::as_str).unwrap_or("Unknown"),
            "languages": languages,
            "currencies": currencies,
            "region": country_data.get("region").and_then(Value::as_str).unwrap_or(""),
            "subregion": country_data.get("subregion").and_then(Value::as_str).unwrap_or(""),
            "area_km2": country_data.get("area").cloned().unwrap_or(json!(0)),
            "flag_emoji": country_data.get("flag").and_then(Value::as_str).unwrap_or(""),
        },
        "local": {
            "name": city_name,
            "state": state_name,
            "description": local_data.get("description").and_then(Value::as_str).unwrap_or(""),
            "summary": local_data.get("extract").and_then(Value::as_str).unwrap_or(""),
            "thumbnail": local_data.get("thumbnail").and_then(|v| v.get("source")).and_then(Value::as_str).unwrap_or(""),
        }
    }))
}

async fn reverse_geocode(client: &reqwest::Client, lat: f64, lng: f64) -> Result<Value> {
    let url = format!(
        "https://nominatim.openstreetmap.org/reverse?lat={}&lon={}&format=json&zoom=10&addressdetails=1&accept-language=en",
        lat, lng
    );
    let resp = client.get(url).header("User-Agent", "Graviton/1.0").send().await?;
    if !resp.status().is_success() {
        return Ok(json!({}));
    }
    let data = resp.json::<Value>().await.unwrap_or_else(|_| json!({}));
    let addr = data.get("address").cloned().unwrap_or_else(|| json!({}));
    Ok(json!({
        "city": addr.get("city").or_else(|| addr.get("town")).or_else(|| addr.get("village")).or_else(|| addr.get("county")).and_then(Value::as_str).unwrap_or(""),
        "state": addr.get("state").or_else(|| addr.get("region")).and_then(Value::as_str).unwrap_or(""),
        "country": addr.get("country").and_then(Value::as_str).unwrap_or(""),
        "country_code": addr.get("country_code").and_then(Value::as_str).unwrap_or("").to_uppercase(),
        "display_name": data.get("display_name").and_then(Value::as_str).unwrap_or(""),
    }))
}

async fn fetch_country_data(client: &reqwest::Client, country_code: &str) -> Result<Value> {
    if country_code.is_empty() {
        return Ok(json!({}));
    }
    let url = format!("https://restcountries.com/v3.1/alpha/{}?fields=name,population,capital,languages,region,subregion,area,currencies,borders,flag", country_code);
    let resp = client.get(url).send().await?;
    if !resp.status().is_success() {
        return Ok(json!({}));
    }
    let data = resp.json::<Value>().await.unwrap_or_else(|_| json!([]));
    Ok(data.as_array().and_then(|a| a.first().cloned()).unwrap_or_else(|| json!({})))
}

async fn fetch_wikidata_leader(client: &reqwest::Client, country_name: &str) -> Result<Value> {
    if country_name.is_empty() {
        return Ok(json!({}));
    }
    let safe_name = country_name.replace('"', "\\\"");
    let query = format!(
        "SELECT ?leaderLabel ?govTypeLabel WHERE {{ ?country wdt:P31 wd:Q6256 ; rdfs:label \"{}\"@en . OPTIONAL {{ ?country wdt:P35 ?leader . }} OPTIONAL {{ ?country wdt:P122 ?govType . }} SERVICE wikibase:label {{ bd:serviceParam wikibase:language \"en\". }} }} LIMIT 1",
        safe_name
    );
    let url = format!("https://query.wikidata.org/sparql?query={}&format=json", urlencoding::encode(&query));
    let resp = client.get(url).header("User-Agent", "Graviton/1.0").send().await?;
    if !resp.status().is_success() {
        return Ok(json!({}));
    }
    let data = resp.json::<Value>().await.unwrap_or_else(|_| json!({}));
    let result = data
        .get("results")
        .and_then(|v| v.get("bindings"))
        .and_then(Value::as_array)
        .and_then(|a| a.first())
        .cloned()
        .unwrap_or_else(|| json!({}));
    Ok(json!({
        "leader": result.get("leaderLabel").and_then(|v| v.get("value")).and_then(Value::as_str).unwrap_or("Unknown"),
        "government_type": result.get("govTypeLabel").and_then(|v| v.get("value")).and_then(Value::as_str).unwrap_or("Unknown"),
    }))
}

async fn fetch_wiki_summary(client: &reqwest::Client, place_name: &str, country_name: &str) -> Result<Value> {
    for candidate in [place_name, &format!("{}, {}", place_name, country_name)] {
        if candidate.trim().is_empty() {
            continue;
        }
        let slug = candidate.replace(' ', "_");
        let url = format!("https://en.wikipedia.org/api/rest_v1/page/summary/{}", urlencoding::encode(&slug));
        let resp = client.get(url).send().await?;
        if !resp.status().is_success() {
            continue;
        }
        let data = resp.json::<Value>().await.unwrap_or_else(|_| json!({}));
        if data.get("type").and_then(Value::as_str) != Some("disambiguation") {
            return Ok(data);
        }
    }
    Ok(json!({}))
}

