'use client'

import { useState, useCallback, useRef, useEffect } from 'react'
import dynamic from 'next/dynamic'
import { Toolbar } from '@/components/Toolbar'
import { LeftPanel } from '@/components/LeftPanel'
import { RightPanel } from '@/components/RightPanel'
import { StatusBar } from '@/components/StatusBar'
import { useDataPolling } from '@/hooks/useDataPolling'

const MapView = dynamic(() => import('@/components/MapView'), { ssr: false })

export interface LayerVisibility {
  commercial_flights: boolean
  military_flights: boolean
  ships: boolean
  satellites: boolean
  earthquakes: boolean
  news: boolean
  firms_fires: boolean
  gdelt: boolean
  internet_outages: boolean
  kiwisdr: boolean
  weather_radar: boolean
  day_night: boolean
}

const DEFAULT_LAYERS: LayerVisibility = {
  commercial_flights: true,
  military_flights: true,
  ships: true,
  satellites: true,
  earthquakes: true,
  news: true,
  firms_fires: false,
  gdelt: false,
  internet_outages: false,
  kiwisdr: false,
  weather_radar: false,
  day_night: false,
}

export default function Dashboard() {
  const [leftOpen, setLeftOpen] = useState(true)
  const [rightOpen, setRightOpen] = useState(true)
  const [layers, setLayers] = useState<LayerVisibility>(DEFAULT_LAYERS)
  const [selectedEntity, setSelectedEntity] = useState<any>(null)
  const [mouseCoords, setMouseCoords] = useState<{ lat: number; lng: number } | null>(null)
  const [zoom, setZoom] = useState(3)

  const { fastData, slowData, isLoading } = useDataPolling()

  const toggleLayer = useCallback((key: keyof LayerVisibility) => {
    setLayers(prev => ({ ...prev, [key]: !prev[key] }))
  }, [])

  const counts = {
    commercial_flights: fastData?.commercial_flights?.length ?? 0,
    military_flights: fastData?.military_flights?.length ?? 0,
    ships: fastData?.ships?.length ?? 0,
    satellites: fastData?.satellites?.length ?? 0,
    earthquakes: slowData?.earthquakes?.length ?? 0,
    news: slowData?.news?.length ?? 0,
    firms_fires: slowData?.firms_fires?.length ?? 0,
    gdelt: slowData?.gdelt?.length ?? 0,
  }

  return (
    <div className="h-screen w-screen flex flex-col overflow-hidden" style={{ background: 'var(--bg-primary)' }}>
      <Toolbar
        leftOpen={leftOpen}
        rightOpen={rightOpen}
        onToggleLeft={() => setLeftOpen(v => !v)}
        onToggleRight={() => setRightOpen(v => !v)}
        isLoading={isLoading}
      />

      <div className="flex flex-1 overflow-hidden">
        {leftOpen && (
          <LeftPanel
            layers={layers}
            counts={counts}
            onToggle={toggleLayer}
          />
        )}

        <div className="flex-1 relative">
          <MapView
            fastData={fastData}
            slowData={slowData}
            layers={layers}
            onSelect={setSelectedEntity}
            onMouseMove={setMouseCoords}
            onZoomChange={setZoom}
          />
        </div>

        {rightOpen && (
          <RightPanel
            selectedEntity={selectedEntity}
            news={slowData?.news ?? []}
            stocks={slowData?.stocks ?? []}
            oil={slowData?.oil ?? []}
          />
        )}
      </div>

      <StatusBar
        coords={mouseCoords}
        zoom={zoom}
        counts={counts}
        spaceWeather={slowData?.space_weather}
      />
    </div>
  )
}
