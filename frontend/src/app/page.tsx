'use client'

import { useMemo, useState } from 'react'
import dynamic from 'next/dynamic'
import { Menu, Settings2, X } from 'lucide-react'
import { LeftPanel } from '@/components/LeftPanel'
import { RightPanel } from '@/components/RightPanel'
import { StatusBar } from '@/components/StatusBar'
import { Toolbar } from '@/components/Toolbar'
import { FilterPanel, type FilterState } from '@/components/FilterPanel'
import { FindLocateBar } from '@/components/FindLocateBar'
import { MapLegend } from '@/components/MapLegend'
import { SettingsPanel } from '@/components/SettingsPanel'
import { RadioInterceptPanel } from '@/components/RadioInterceptPanel'
import { useDataPolling } from '@/hooks/useDataPolling'
import { DashboardDataProvider } from '@/lib/DashboardDataContext'

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
  gdelt: true,
  liveuamap: true,
  frontlines: true,
  internet_outages: false,
  kiwisdr: false,
  datacenters: false,
  cctv: false,
  weather_radar: false,
  day_night: true,
}

const DEFAULT_FILTERS: FilterState = {
  militaryTypes: [],
  trackedCategories: [],
  yachtCategories: [],
}

export default function Dashboard() {
  const [leftOpen, setLeftOpen] = useState(true)
  const [rightOpen, setRightOpen] = useState(true)
  const [mobileMenu, setMobileMenu] = useState(false)
  const [settingsOpen, setSettingsOpen] = useState(false)
  const [layers, setLayers] = useState<LayerVisibility>(DEFAULT_LAYERS)
  const [selectedEntity, setSelectedEntity] = useState<any>(null)
  const [mouseCoords, setMouseCoords] = useState<{ lat: number; lng: number } | null>(null)
  const [zoom, setZoom] = useState(2.8)
  const [focusLocation, setFocusLocation] = useState<{ lat: number; lng: number; label?: string } | null>(null)
  const [filters, setFilters] = useState<FilterState>(DEFAULT_FILTERS)
  const { fastData, slowData, isLoading } = useDataPolling()

  const filteredFastData = useMemo(() => {
    if (!fastData) return null
    return {
      ...fastData,
      military_flights:
        filters.militaryTypes.length === 0
          ? fastData.military_flights
          : fastData.military_flights.filter((item: any) => filters.militaryTypes.includes(item.military_type)),
      tracked_flights:
        filters.trackedCategories.length === 0
          ? fastData.tracked_flights
          : fastData.tracked_flights.filter((item: any) => filters.trackedCategories.includes(item.alert_category)),
      ships:
        filters.yachtCategories.length === 0
          ? fastData.ships
          : fastData.ships.filter((item: any) => filters.yachtCategories.includes(item.yacht_category)),
    }
  }, [fastData, filters])

  const counts = useMemo(
    () => ({
      commercial_flights: filteredFastData?.commercial_flights?.length ?? 0,
      private_flights: filteredFastData?.private_flights?.length ?? 0,
      private_jets: filteredFastData?.private_jets?.length ?? 0,
      military_flights: filteredFastData?.military_flights?.length ?? 0,
      tracked_flights: filteredFastData?.tracked_flights?.length ?? 0,
      uavs: filteredFastData?.uavs?.length ?? 0,
      gps_jamming: filteredFastData?.gps_jamming?.length ?? 0,
      ships: filteredFastData?.ships?.length ?? 0,
      satellites: filteredFastData?.satellites?.length ?? 0,
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
    [filteredFastData, slowData],
  )

  const dashboardData = { fastData: filteredFastData, slowData }

  return (
    <DashboardDataProvider fastData={filteredFastData} slowData={slowData} selectedEntity={selectedEntity} setSelectedEntity={setSelectedEntity}>
      <div className="flex h-screen w-screen flex-col overflow-hidden bg-[var(--bg-primary)] text-[var(--text-primary)]">
        <Toolbar
          leftOpen={leftOpen}
          rightOpen={rightOpen}
          onToggleLeft={() => setLeftOpen((v) => !v)}
          onToggleRight={() => setRightOpen((v) => !v)}
          onSearchSelect={setFocusLocation}
          isLoading={isLoading}
        />

        <div className="flex items-center justify-between gap-3 border-b border-white/10 bg-black/25 px-3 py-2 md:hidden">
          <button onClick={() => setMobileMenu((v) => !v)} className="rounded-xl border border-white/10 p-2 text-slate-200">
            {mobileMenu ? <X size={16} /> : <Menu size={16} />}
          </button>
          <div className="min-w-0 flex-1">
            <FindLocateBar onSelect={setFocusLocation} />
          </div>
          <button onClick={() => setSettingsOpen(true)} className="rounded-xl border border-white/10 p-2 text-slate-200">
            <Settings2 size={16} />
          </button>
        </div>

        <div className="hidden items-start gap-3 border-b border-white/10 bg-black/15 px-3 py-3 md:flex">
          <FindLocateBar onSelect={setFocusLocation} />
          <button onClick={() => setSettingsOpen(true)} className="rounded-2xl border border-white/10 bg-black/40 px-4 py-2 text-sm text-slate-200 hover:bg-white/5">
            Settings
          </button>
          <div className="grid flex-1 grid-cols-1 gap-3 lg:grid-cols-[minmax(0,1fr)_20rem_20rem]">
            <MapLegend layers={layers} />
            <FilterPanel data={dashboardData} filters={filters} onChange={setFilters} />
            <RadioInterceptPanel location={focusLocation} />
          </div>
        </div>

        {mobileMenu && (
          <div className="border-b border-white/10 bg-[#07111b] px-3 py-3 md:hidden">
            <div className="grid gap-3">
              <MapLegend layers={layers} />
              <FilterPanel data={dashboardData} filters={filters} onChange={setFilters} />
              <RadioInterceptPanel location={focusLocation} />
            </div>
          </div>
        )}

        <div className="flex min-h-0 flex-1">
          <div className={`${leftOpen ? 'hidden lg:block' : 'hidden'} shrink-0`}>
            <LeftPanel layers={layers} counts={counts} onToggle={(key) => setLayers((prev) => ({ ...prev, [key]: !prev[key] }))} />
          </div>

          <MapView
            data={dashboardData}
            layers={layers}
            focusLocation={focusLocation}
            onSelect={setSelectedEntity}
            onMouseMove={setMouseCoords}
            onZoomChange={setZoom}
          />

          <div className={`${rightOpen ? 'hidden xl:block' : 'hidden'} shrink-0`}>
            <RightPanel selectedEntity={selectedEntity} fastData={filteredFastData} slowData={slowData} focusLocation={focusLocation} />
          </div>
        </div>

        <StatusBar coords={mouseCoords} zoom={zoom} counts={counts} spaceWeather={slowData?.space_weather} />
        <SettingsPanel open={settingsOpen} onClose={() => setSettingsOpen(false)} />
      </div>
    </DashboardDataProvider>
  )
}
