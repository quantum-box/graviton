use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Aircraft {
    pub icao: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub callsign: Option<String>,
    pub lat: f64,
    pub lng: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub altitude: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heading: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speed: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub registration: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aircraft_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operator: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub on_ground: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub squawk: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trail: Option<Vec<[f64; 2]>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ship {
    pub mmsi: String,
    pub lat: f64,
    pub lng: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heading: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speed: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ship_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destination: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cargo: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub length: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flag: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Satellite {
    pub norad_id: u32,
    pub name: String,
    pub lat: f64,
    pub lng: f64,
    pub altitude_km: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speed_km_s: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heading: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mission: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Earthquake {
    pub id: String,
    pub lat: f64,
    pub lng: f64,
    pub magnitude: f64,
    pub depth_km: f64,
    pub place: String,
    pub time: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsItem {
    pub id: String,
    pub title: String,
    pub source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lat: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lng: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub risk_score: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FireHotspot {
    pub lat: f64,
    pub lng: f64,
    pub frp: f64,
    pub confidence: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub satellite: Option<String>,
    pub acq_time: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GdeltEvent {
    pub lat: f64,
    pub lng: f64,
    pub event_code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub goldstein_scale: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_mentions: Option<u32>,
    pub date: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InternetOutage {
    pub region: String,
    pub country: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lat: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lng: Option<f64>,
    pub severity: f64,
    pub datasource: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataCenter {
    pub id: String,
    pub name: String,
    pub lat: f64,
    pub lng: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub company: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub city: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CctvCamera {
    pub id: String,
    pub lat: f64,
    pub lng: f64,
    pub source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub direction: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KiwiSdr {
    pub name: String,
    pub lat: f64,
    pub lng: f64,
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub antenna: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bands: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub users: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub users_max: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockData {
    pub symbol: String,
    pub price: f64,
    pub change_pct: f64,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpaceWeather {
    pub kp_index: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub solar_events: Option<Vec<String>>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpsJamming {
    pub lat: f64,
    pub lng: f64,
    pub intensity: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub area_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Airport {
    pub icao: String,
    pub name: String,
    pub lat: f64,
    pub lng: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iata: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionDossier {
    pub country_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flag: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capital: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub population: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub leader: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wiki_summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub languages: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currencies: Option<Vec<String>>,
}
