'use client'

import type { LayerVisibility } from '@/app/page'
import type { DashboardData, FocusLocation } from '@/types/dashboard'
import { computeNightPolygon } from '@/utils/solarTerminator'
import { getFlightOperator } from '@/utils/airlineCodes'
import { useEffect, useMemo, useRef, useState } from 'react'
import Map, { Layer, MapLayerMouseEvent, MapRef, Popup, Source } from 'react-map-gl/maplibre'
import 'maplibre-gl/dist/maplibre-gl.css'
import { MapMarkers } from '@/components/map/MapMarkers'
import { ScaleBar } from '@/components/ScaleBar'

export default function MapView({
  data,
  layers,
  zoom,
  focusLocation,
  onSelect,
  onMouseMove,
  onZoomChange,
  onViewChange,
  activeTool,
  trajectory,
  prediction,
}: {
  data: DashboardData
  layers: LayerVisibility
  zoom: number
  focusLocation: FocusLocation | null
  onSelect: (entity: any) => void
  onMouseMove: (coords: { lat: number; lng: number } | null) => void
  onZoomChange: (zoom: number) => void
  onViewChange?: (view: { lat: number; lng: number; zoom: number }) => void
  activeTool?: string
  trajectory?: Array<{ lat: number; lng: number }>
  prediction?: Array<{ lat: number; lng: number }>
}) {
  const mapRef = useRef<MapRef>(null)
  const [popup, setPopup] = useState<any>(null)
  const [mapCenter, setMapCenter] = useState({ lat: 26, lng: 20 })
  const nightPolygon = useMemo(() => computeNightPolygon(), [data.fastData?.last_updated, data.slowData?.last_updated])

  useEffect(() => {
    if (focusLocation && mapRef.current) {
      mapRef.current.flyTo({ center: [focusLocation.lng, focusLocation.lat], zoom: 6.5, duration: 1400 })
    }
  }, [focusLocation])

  const interactiveLayerIds = useMemo(
    () =>
      [
        'commercial_flights-points',
        'private_flights-points',
        'private_jets-points',
        'military_flights-points',
        'tracked_flights-points',
        'uavs-points',
        'gps_jamming-points',
        'ships-points',
        'satellites-points',
        'earthquakes-points',
        'news-points',
        'firms_fires-points',
        'gdelt-points',
        'liveuamap-points',
        'internet_outages-points',
        'kiwisdr-points',
        'datacenters-points',
        'cctv-points',
      ],
    [],
  )

  function handleClick(event: MapLayerMouseEvent) {
    const feature = event.features?.find((item) => !(item.properties as any)?.point_count)
    if (!feature) return
    const props = feature.properties ?? {}
    const selected = { ...props, lat: event.lngLat.lat, lng: event.lngLat.lng, sourceType: props.__kind }
    setPopup(selected)
    onSelect(selected)
  }

  return (
    <div
      className={`fixed inset-0 h-[100dvh] w-screen overflow-hidden ${activeTool === 'measure' ? 'cursor-crosshair' : activeTool === 'marker' ? 'cursor-cell' : activeTool === 'filter' ? 'cursor-help' : 'cursor-default'}`}
      data-map-view="true"
    >
      <Map
        ref={mapRef}
        reuseMaps
        initialViewState={{ longitude: 20, latitude: 26, zoom: 2.8 }}
        mapStyle="https://basemaps.cartocdn.com/gl/dark-matter-gl-style/style.json"
        interactiveLayerIds={interactiveLayerIds}
        style={{ width: '100%', height: '100%' }}
        attributionControl={false}
        dragPan
        touchZoomRotate
        onClick={handleClick}
        onMouseMove={(e) => onMouseMove({ lat: e.lngLat.lat, lng: e.lngLat.lng })}
        onZoomEnd={(e) => onZoomChange(e.viewState.zoom)}
        onMove={(e) => {
          const nextView = { lat: e.viewState.latitude, lng: e.viewState.longitude, zoom: e.viewState.zoom }
          setMapCenter({ lat: nextView.lat, lng: nextView.lng })
          onViewChange?.(nextView)
        }}
      >
        {layers.weather_radar && data.slowData?.weather && (data.slowData.weather as any).host && (data.slowData.weather as any).radar_tile_path && (
          <Source
            id="weather-radar"
            type="raster"
            tiles={[`${(data.slowData.weather as any).host}${(data.slowData.weather as any).radar_tile_path}/256/{z}/{x}/{y}/6/1_1.png`]}
            tileSize={256}
          >
            <Layer id="weather-radar" type="raster" paint={{ 'raster-opacity': 0.5 }} />
          </Source>
        )}

        {layers.sentinel_overlay && (
          <Source
            id="sentinel-overlay"
            type="raster"
            tiles={['https://tiles.maps.eox.at/wmts/1.0.0/s2cloudless-2024_3857/default/g/{z}/{y}/{x}.jpg']}
            tileSize={256}
          >
            <Layer id="sentinel-overlay" type="raster" paint={{ 'raster-opacity': 0.38 }} />
          </Source>
        )}

        {layers.frontlines && data.slowData?.frontlines && (
          <Source id="frontlines" type="geojson" data={data.slowData.frontlines as any}>
            <Layer id="frontlines-fill" type="fill" paint={{ 'fill-color': '#facc15', 'fill-opacity': 0.1 }} />
            <Layer id="frontlines-line" type="line" paint={{ 'line-color': '#facc15', 'line-width': 1.2 }} />
          </Source>
        )}

        {layers.day_night && (
          <Source id="night-polygon" type="geojson" data={nightPolygon as any}>
            <Layer id="night-polygon-fill" type="fill" paint={{ 'fill-color': '#020617', 'fill-opacity': 0.28 }} />
          </Source>
        )}

        <MapMarkers data={data} layers={layers} />

        {layers.trajectories && trajectory && trajectory.length > 1 && (
          <Source
            id="trajectory-line"
            type="geojson"
            data={{
              type: 'Feature',
              geometry: {
                type: 'LineString',
                coordinates: trajectory.map((point) => [point.lng, point.lat]),
              },
              properties: {},
            }}
          >
            <Layer id="trajectory-line" type="line" paint={{ 'line-color': '#fbbf24', 'line-width': 2.5, 'line-opacity': 0.95 }} />
          </Source>
        )}

        {layers.predictions && prediction && prediction.length > 1 && (
          <Source
            id="prediction-line"
            type="geojson"
            data={{
              type: 'Feature',
              geometry: {
                type: 'LineString',
                coordinates: prediction.map((point) => [point.lng, point.lat]),
              },
              properties: {},
            }}
          >
            <Layer id="prediction-line" type="line" paint={{ 'line-color': '#f472b6', 'line-width': 2, 'line-dasharray': [1, 1], 'line-opacity': 0.95 }} />
          </Source>
        )}

        {popup && (
          <Popup longitude={popup.lng} latitude={popup.lat} onClose={() => setPopup(null)} closeButton>
            <div className="space-y-1 text-xs">
              <div className="font-semibold text-cyan-300">{popup.title || popup.name || popup.callsign || popup.icao24 || popup.sourceType}</div>
              {getFlightOperator(popup) && <div>{getFlightOperator(popup)}</div>}
              {popup.country && <div>{popup.country}</div>}
              {popup.description && <div className="max-w-[240px] text-slate-300">{String(popup.description).replace(/<[^>]+>/g, '').slice(0, 180)}</div>}
            </div>
          </Popup>
        )}
      </Map>
      <ScaleBar zoom={zoom} latitude={mapCenter.lat} />
    </div>
  )
}
