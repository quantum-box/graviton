'use client'

import { Layer, Source } from 'react-map-gl/maplibre'
import type { LayerVisibility } from '@/app/page'
import type { BaseEntity, DashboardData } from '@/types/dashboard'
import { pointCollection } from '@/components/map/geoJSONBuilders'
import { useInterpolation } from '@/components/map/hooks/useInterpolation'

const STYLE_MAP: Record<string, any> = {
  commercial_flights: { color: '#38bdf8', radius: 4 },
  private_flights: { color: '#818cf8', radius: 4 },
  private_jets: { color: '#f472b6', radius: 5 },
  military_flights: { color: '#ef4444', radius: 5 },
  tracked_flights: { color: '#ec4899', radius: 6 },
  uavs: { color: '#f59e0b', radius: 5 },
  gps_jamming: { color: '#f97316', radius: 8 },
  ships: { color: '#22c55e', radius: 5 },
  satellites: { color: '#fde047', radius: 4 },
  earthquakes: { color: '#fb923c', radius: 5 },
  news: { color: '#a78bfa', radius: 5 },
  firms_fires: { color: '#f97316', radius: 5 },
  gdelt: { color: '#f43f5e', radius: 4 },
  liveuamap: { color: '#fb7185', radius: 4 },
  internet_outages: { color: '#14b8a6', radius: 5 },
  kiwisdr: { color: '#67e8f9', radius: 4 },
  datacenters: { color: '#94a3b8', radius: 4 },
  cctv: { color: '#c084fc', radius: 4 },
}

export function MapMarkers({ data, layers }: { data: DashboardData; layers: LayerVisibility }) {
  const commercialFlights = useInterpolation(data.fastData?.commercial_flights ?? [])
  const privateFlights = useInterpolation(data.fastData?.private_flights ?? [])
  const privateJets = useInterpolation(data.fastData?.private_jets ?? [])
  const militaryFlights = useInterpolation(data.fastData?.military_flights ?? [])
  const trackedFlights = useInterpolation(data.fastData?.tracked_flights ?? [])
  const uavs = useInterpolation(data.fastData?.uavs ?? [])
  const ships = useInterpolation(data.fastData?.ships ?? [])
  const satellites = useInterpolation(data.fastData?.satellites ?? [])
  const collections: Record<string, GeoJSON.FeatureCollection> = {
    commercial_flights: pointCollection(commercialFlights, 'commercial_flights'),
    private_flights: pointCollection(privateFlights, 'private_flights'),
    private_jets: pointCollection(privateJets, 'private_jets'),
    military_flights: pointCollection(militaryFlights, 'military_flights'),
    tracked_flights: pointCollection(trackedFlights, 'tracked_flights'),
    uavs: pointCollection(uavs, 'uavs'),
    gps_jamming: pointCollection(data.fastData?.gps_jamming ?? [], 'gps_jamming'),
    ships: pointCollection(ships, 'ships'),
    satellites: pointCollection(satellites, 'satellites'),
    earthquakes: pointCollection(data.slowData?.earthquakes ?? [], 'earthquakes'),
    news: pointCollection(data.slowData?.news ?? [], 'news'),
    firms_fires: pointCollection(data.slowData?.firms_fires ?? [], 'firms_fires'),
    gdelt: pointCollection(data.slowData?.gdelt ?? [], 'gdelt'),
    liveuamap: pointCollection(data.slowData?.liveuamap ?? [], 'liveuamap'),
    internet_outages: pointCollection(data.slowData?.internet_outages ?? [], 'internet_outages'),
    kiwisdr: pointCollection(data.slowData?.kiwisdr ?? [], 'kiwisdr'),
    datacenters: pointCollection(data.slowData?.datacenters ?? [], 'datacenters'),
    cctv: pointCollection(data.slowData?.cctv ?? [], 'cctv'),
  }

  return (
    <>
      {Object.entries(collections).map(([key, collection]) => {
        if (!layers[key as keyof LayerVisibility]) return null
        const style = STYLE_MAP[key]
        return (
          <Source key={key} id={key} type="geojson" data={collection as any} cluster clusterRadius={50}>
            <Layer id={`${key}-clusters`} type="circle" filter={['has', 'point_count']} paint={{ 'circle-color': style.color, 'circle-radius': 14, 'circle-opacity': 0.35 }} />
            <Layer id={`${key}-cluster-count`} type="symbol" filter={['has', 'point_count']} layout={{ 'text-field': ['get', 'point_count_abbreviated'], 'text-size': 11 }} paint={{ 'text-color': '#f8fafc' }} />
            <Layer id={`${key}-points`} type="circle" filter={['!', ['has', 'point_count']]} paint={{ 'circle-color': style.color, 'circle-radius': style.radius, 'circle-stroke-width': 1, 'circle-stroke-color': '#081018' }} />
          </Source>
        )
      })}
    </>
  )
}
