'use client'

import { useMemo, useState } from 'react'
import dynamic from 'next/dynamic'
import { Toolbar } from '@/components/Toolbar'
import { LeftPanel } from '@/components/LeftPanel'
import { RightPanel } from '@/components/RightPanel'
import { StatusBar } from '@/components/StatusBar'
import { useDataPolling } from '@/hooks/useDataPolling'

const MapView = dynamic(() => import('@/components/MapView'), { ssr: false })

export interface LayerVisibility {
  commercial_flights: boolean
  private_flights: boolean
  private_jets: boolean
  military_flights: boolean
  tracked_flights: boolean
  uavs: boolean
  gps_jamming: boolean
  ships: boolean
  satellites: boolean
  earthquakes: boolean
  news: boolean
  firms_fires: boolean
  gdelt: boolean
  liveuamap: boolean
  frontlines: boolean
  internet_outages: boolean
  kiwisdr: boolean
  datacenters: boolean
  cctv: boolean
  weather_radar: boolean
  day_night: boolean
}

const DEFAULT_LAYERS: LayerVisibility = {
  commercial_flights: true,
  private_flights: true,
  private_jets: true,
  military_flights: true,
  tracked_flights: true,
  uavs: true,
  gps_jamming: true,
  ships: true,
  satellites: true,
  earthquakes: true,
  news: true,
  firms_fires: false,
  gdelt: false,
  liveuamap: false,
  frontlines: true,
  internet_outages: false,
  kiwisdr: false,
  datacenters: false,
  cctv: false,
  weather_radar: false,
  day_night: false,
}

export default function Dashboard() {
  const [leftOpen, setLeftOpen] = useState(true)
  const [rightOpen, setRightOpen] = useState(true)
  const [layers, setLayers] = useState<LayerVisibility>(DEFAULT_LAYERS)
  const [selectedEntity, setSelectedEntity] = useState<any>(null)
  const [mouseCoords, setMouseCoords] = useState<{ lat: number; lng: number } | null>(null)
  const [zoom, setZoom] = useState(2.8)
  const [focusLocation, setFocusLocation] = useState<{ lat: number; lng: number; label?: string } | null>(null)

  const { fastData, slowData, isLoading } = useDataPolling()

  const counts = useMemo(
    () => ({
      commercial_flights: fastData?.commercial_flights?.length ?? 0,
      private_flights: fastData?.private_flights?.length ?? 0,
      private_jets: fastData?.private_jets?.length ?? 0,
      military_flights: fastData?.military_flights?.length ?? 0,
      tracked_flights: fastData?.tracked_flights?.length ?? 0,
      uavs: fastData?.uavs?.length ?? 0,
      gps_jamming: fastData?.gps_jamming?.length ?? 0,
      ships: fastData?.ships?.length ?? 0,
      satellites: fastData?.satellites?.length ?? 0,
      earthquakes: slowData?.earthquakes?.length ?? 0,
      news: slowData?.news?.length ?? 0,
      firms_fires: slowData?.firms_fires?.length ?? 0,
      gdelt: slowData?.gdelt?.length ?? 0,
      liveuamap: slowData?.liveuamap?.length ?? 0,
      frontlines: slowData?.frontlines?.features?.length ?? 0,
      internet_outages: slowData?.internet_outages?.length ?? 0,
      kiwisdr: slowData?.kiwisdr?.length ?? 0,
      datacenters: slowData?.datacenters?.length ?? 0,
      cctv: slowData?.cctv?.length ?? 0,
    }),
    [fastData, slowData],
  )

  return (
    <div className="h-screen w-screen overflow-hidden bg-[var(--bg-primary)] text-[var(--text-primary)]">
      <Toolbar
        leftOpen={leftOpen}
        rightOpen={rightOpen}
        onToggleLeft={() => setLeftOpen((v) => !v)}
        onToggleRight={() => setRightOpen((v) => !v)}
        onSearchSelect={setFocusLocation}
        isLoading={isLoading}
      />

      <div className="flex h-[calc(100vh-var(--toolbar-height)-var(--statusbar-height))]">
        {leftOpen && <LeftPanel layers={layers} counts={counts} onToggle={(key) => setLayers((prev) => ({ ...prev, [key]: !prev[key] }))} />}
        <MapView
          fastData={fastData}
          slowData={slowData}
          layers={layers}
          focusLocation={focusLocation}
          onSelect={setSelectedEntity}
          onMouseMove={setMouseCoords}
          onZoomChange={setZoom}
        />
        {rightOpen && <RightPanel selectedEntity={selectedEntity} fastData={fastData} slowData={slowData} focusLocation={focusLocation} />}
      </div>

      <StatusBar coords={mouseCoords} zoom={zoom} counts={counts} spaceWeather={slowData?.space_weather} />
    </div>
  )
}
