# Graviton

Real-time geospatial intelligence dashboard. Aggregates 30+ open-source data feeds onto a unified map interface.

## Architecture

- **Backend**: Rust (Axum) — concurrent data fetchers, in-memory store, REST API
- **Frontend**: Next.js + MapLibre GL — Mac-style desktop UI with dark theme

## Data Sources

| Category | Sources |
|----------|---------|
| Aircraft | adsb.lol (military + commercial), OpenSky Network |
| Ships | AIS Stream (WebSocket), carrier OSINT tracker |
| Satellites | CelesTrak TLEs + SGP4 propagation |
| Earthquakes | USGS real-time feed |
| Fires | NASA FIRMS (NOAA-20 VIIRS) |
| News | RSS feeds (NPR, BBC, Al Jazeera, NYT, GDACS) |
| Conflicts | GDELT Global Knowledge Graph |
| Financial | Yahoo Finance (defense stocks, oil) |
| Space Weather | NOAA SWPC Kp index |
| Weather | RainViewer radar |
| Infrastructure | IODA internet outages, KiwiSDR receivers |

## Quick Start

```bash
# Docker
docker compose up --build

# Development
cd backend && cargo run &
cd frontend && npm install && npm run dev
```

Backend: http://localhost:8000
Frontend: http://localhost:3000

## Environment Variables

Copy `.env.example` to `.env` and configure API keys.

| Variable | Required | Description |
|----------|----------|-------------|
| `AIS_API_KEY` | Optional | aisstream.io WebSocket key |
| `OPENSKY_CLIENT_ID` | Optional | OpenSky Network OAuth2 |
| `OPENSKY_CLIENT_SECRET` | Optional | OpenSky Network OAuth2 |
| `ADMIN_KEY` | Optional | Admin auth for settings API |

## API Endpoints

### Fast Tier (15s polling)
- `GET /api/live-data/fast` — flights, ships, satellites

### Slow Tier (120s polling)
- `GET /api/live-data/slow` — news, earthquakes, fires, weather, GDELT

### Intelligence
- `GET /api/region-dossier?lat=&lng=` — country profile on right-click
- `GET /api/route/{callsign}` — flight route lookup
- `GET /api/sentinel2/search?lat=&lng=` — Sentinel-2 imagery

### Radio
- `GET /api/radio/top` — Broadcastify top feeds
- `GET /api/radio/openmhz/systems` — scanner systems

### System
- `GET /api/health` — source status + uptime
- `POST /api/refresh` — force data refresh
