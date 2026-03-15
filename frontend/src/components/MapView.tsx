'use client'

import type { LayerVisibility } from '@/app/page'
import { useEffect, useMemo, useRef, useState } from 'react'
import Map, { Layer, MapLayerMouseEvent, MapRef, Popup, Source } from 'react-map-gl/maplibre'
import 'maplibre-gl/dist/maplibre-gl.css'

function fc(items: any[], kind: string, getCoords: (item: any) => [number, number] | null) {
  return {
    type: 'FeatureCollection',
    features: items
      .map((item) => {
        const coords = getCoords(item)
        if (!coords) return null
        return {
          type: 'Feature',
          properties: { ...item, __kind: kind },
          geometry: { type: 'Point', coordinates: coords },
        }
      })
      .filter(Boolean),
  }
}

const LAYER_STYLES: Record<string, any> = {
  commercial_flights: { id: 'commercial_flights', type: 'circle', paint: { 'circle-radius': 3, 'circle-color': '#38bdf8', 'circle-opacity': 0.85 } },
  private_flights: { id: 'private_flights', type: 'circle', paint: { 'circle-radius': 3, 'circle-color': '#818cf8', 'circle-opacity': 0.75 } },
  private_jets: { id: 'private_jets', type: 'circle', paint: { 'circle-radius': 4, 'circle-color': '#f472b6' } },
  military_flights: { id: 'military_flights', type: 'circle', paint: { 'circle-radius': 4, 'circle-color': '#ef4444' } },
  tracked_flights: { id: 'tracked_flights', type: 'circle', paint: { 'circle-radius': 5, 'circle-color': '#ec4899' } },
  uavs: { id: 'uavs', type: 'circle', paint: { 'circle-radius': 4, 'circle-color': '#f59e0b' } },
  gps_jamming: { id: 'gps_jamming', type: 'circle', paint: { 'circle-radius': 7, 'circle-color': '#f97316', 'circle-opacity': 0.35, 'circle-stroke-color': '#fdba74', 'circle-stroke-width': 1 } },
  ships: { id: 'ships', type: 'circle', paint: { 'circle-radius': 4, 'circle-color': '#22c55e' } },
  satellites: { id: 'satellites', type: 'circle', paint: { 'circle-radius': 3, 'circle-color': '#fde047' } },
  earthquakes: { id: 'earthquakes', type: 'circle', paint: { 'circle-radius': 5, 'circle-color': '#fb923c' } },
  news: { id: 'news', type: 'circle', paint: { 'circle-radius': 5, 'circle-color': '#a78bfa' } },
  firms_fires: { id: 'firms_fires', type: 'circle', paint: { 'circle-radius': 4, 'circle-color': '#f97316' } },
  gdelt: { id: 'gdelt', type: 'circle', paint: { 'circle-radius': 4, 'circle-color': '#f43f5e' } },
  liveuamap: { id: 'liveuamap', type: 'circle', paint: { 'circle-radius': 4, 'circle-color': '#fb7185' } },
  internet_outages: { id: 'internet_outages', type: 'circle', paint: { 'circle-radius': 5, 'circle-color': '#14b8a6' } },
  kiwisdr: { id: 'kiwisdr', type: 'circle', paint: { 'circle-radius': 4, 'circle-color': '#67e8f9' } },
  datacenters: { id: 'datacenters', type: 'circle', paint: { 'circle-radius': 3, 'circle-color': '#94a3b8' } },
  cctv: { id: 'cctv', type: 'circle', paint: { 'circle-radius': 3, 'circle-color': '#c084fc' } },
}

interface MapViewProps {
  fastData: any
  slowData: any
  layers: LayerVisibility
  focusLocation: { lat: number; lng: number; label?: string } | null
  onSelect: (entity: any) => void
  onMouseMove: (coords: { lat: number; lng: number } | null) => void
  onZoomChange: (zoom: number) => void
}

export default function MapView({ fastData, slowData, layers, focusLocation, onSelect, onMouseMove, onZoomChange }: MapViewProps) {
  const mapRef = useRef<MapRef>(null)
  const [popup, setPopup] = useState<any>(null)

  useEffect(() => {
    if (focusLocation && mapRef.current) {
      mapRef.current.flyTo({ center: [focusLocation.lng, focusLocation.lat], zoom: 7, duration: 1800 })
    }
  }, [focusLocation])

  const sources = useMemo(
    () => ({
      commercial_flights: fc(fastData?.commercial_flights ?? [], 'commercial_flights', (v) => [v.lng, v.lat]),
      private_flights: fc(fastData?.private_flights ?? [], 'private_flights', (v) => [v.lng, v.lat]),
      private_jets: fc(fastData?.private_jets ?? [], 'private_jets', (v) => [v.lng, v.lat]),
      military_flights: fc(fastData?.military_flights ?? [], 'military_flights', (v) => [v.lng, v.lat]),
      tracked_flights: fc(fastData?.tracked_flights ?? [], 'tracked_flights', (v) => [v.lng, v.lat]),
      uavs: fc(fastData?.uavs ?? [], 'uavs', (v) => [v.lng, v.lat]),
      gps_jamming: fc(fastData?.gps_jamming ?? [], 'gps_jamming', (v) => [v.lng, v.lat]),
      ships: fc(fastData?.ships ?? [], 'ships', (v) => [v.lng, v.lat]),
      satellites: fc(fastData?.satellites ?? [], 'satellites', (v) => [v.lng, v.lat]),
      earthquakes: fc(slowData?.earthquakes ?? [], 'earthquakes', (v) => [v.lng, v.lat]),
      news: fc(slowData?.news ?? [], 'news', (v) => (v.lat && v.lng ? [v.lng, v.lat] : null)),
      firms_fires: fc(slowData?.firms_fires ?? [], 'firms_fires', (v) => [v.lng, v.lat]),
      gdelt: fc(slowData?.gdelt ?? [], 'gdelt', (v) => [v.lng, v.lat]),
      liveuamap: fc(slowData?.liveuamap ?? [], 'liveuamap', (v) => [v.lng, v.lat]),
      internet_outages: fc(slowData?.internet_outages ?? [], 'internet_outages', (v) => (v.lat && v.lng ? [v.lng, v.lat] : null)),
      kiwisdr: fc(slowData?.kiwisdr ?? [], 'kiwisdr', (v) => [v.lng, v.lat]),
      datacenters: fc(slowData?.datacenters ?? [], 'datacenters', (v) => [v.lng, v.lat]),
      cctv: fc(slowData?.cctv ?? [], 'cctv', (v) => [v.lng, v.lat]),
    }),
    [fastData, slowData],
  )

  function onClick(e: MapLayerMouseEvent) {
    const feature = e.features?.[0]
    if (!feature) return
    const props = feature.properties ?? {}
    const selected = { ...props, lat: e.lngLat.lat, lng: e.lngLat.lng, sourceType: props.__kind }
    setPopup(selected)
    onSelect(selected)
  }

  const interactiveLayerIds = (Object.keys(LAYER_STYLES) as Array<keyof LayerVisibility>).filter((key) => layers[key])

  return (
    <div className="relative flex-1">
      <Map
        ref={mapRef}
        initialViewState={{ longitude: 20, latitude: 26, zoom: 2.8 }}
        mapStyle="https://basemaps.cartocdn.com/gl/dark-matter-gl-style/style.json"
        interactiveLayerIds={interactiveLayerIds}
        onClick={onClick}
        onMouseMove={(e) => onMouseMove({ lat: e.lngLat.lat, lng: e.lngLat.lng })}
        onZoomEnd={(e) => onZoomChange(e.viewState.zoom)}
      >
        {layers.weather_radar && slowData?.weather?.host && slowData?.weather?.radar_tile_path && (
          <Source
            id="weather-radar"
            type="raster"
            tiles={[`${slowData.weather.host}${slowData.weather.radar_tile_path}/256/{z}/{x}/{y}/6/1_1.png`]}
            tileSize={256}
          >
            <Layer id="weather-radar" type="raster" paint={{ 'raster-opacity': 0.5 }} />
          </Source>
        )}

        {layers.frontlines && slowData?.frontlines && (
          <Source id="frontlines" type="geojson" data={slowData.frontlines}>
            <Layer id="frontlines-fill" type="fill" paint={{ 'fill-color': '#facc15', 'fill-opacity': 0.12 }} />
            <Layer id="frontlines-line" type="line" paint={{ 'line-color': '#facc15', 'line-width': 1.2 }} />
          </Source>
        )}

        {(Object.entries(sources) as Array<[keyof typeof sources, any]>).map(([key, data]) =>
          layers[key as keyof LayerVisibility] ? (
            <Source key={key} id={key} type="geojson" data={data as any}>
              <Layer {...LAYER_STYLES[key]} />
            </Source>
          ) : null,
        )}

        {popup && (
          <Popup longitude={popup.lng} latitude={popup.lat} onClose={() => setPopup(null)} closeButton>
            <div className="space-y-1 text-xs">
              <div className="font-semibold text-cyan-300">{popup.title || popup.name || popup.callsign || popup.icao24 || popup.region_name || popup.sourceType}</div>
              {popup.operator && <div>{popup.operator}</div>}
              {popup.country && <div>{popup.country}</div>}
              {popup.description && <div className="max-w-[240px] text-slate-300">{String(popup.description).replace(/<[^>]+>/g, '').slice(0, 160)}</div>}
            </div>
          </Popup>
        )}
      </Map>
    </div>
  )
}
