'use client'

import { useMemo, useState } from 'react'
import { ConflictPanel } from '@/components/panels/ConflictPanel'
import { EarthObservationPanel } from '@/components/panels/EarthObservationPanel'
import { FinancialPanel } from '@/components/panels/FinancialPanel'
import { FlightPanel } from '@/components/panels/FlightPanel'
import { GeocodingPanel } from '@/components/panels/GeocodingPanel'
import { InfrastructurePanel } from '@/components/panels/InfrastructurePanel'
import { NewsPanel } from '@/components/panels/NewsPanel'
import { SatellitePanel } from '@/components/panels/SatellitePanel'
import { ShipPanel } from '@/components/panels/ShipPanel'

interface RightPanelProps {
  selectedEntity: any
  fastData: any
  slowData: any
  focusLocation: { lat: number; lng: number; label?: string } | null
}

const TABS = ['selection', 'news', 'markets', 'earth', 'conflicts', 'infra', 'geocode'] as const

export function RightPanel({ selectedEntity, fastData, slowData, focusLocation }: RightPanelProps) {
  const [tab, setTab] = useState<(typeof TABS)[number]>('selection')

  const celebrity = useMemo(
    () => ({
      trackedFlights: fastData?.tracked_flights ?? [],
      yachts: (fastData?.ships ?? []).filter((ship: any) => ship.yacht_alert),
    }),
    [fastData],
  )

  return (
    <aside className="flex w-[360px] shrink-0 flex-col overflow-hidden border-l border-[var(--border-color)] bg-[linear-gradient(180deg,#09131e,#081018)]">
      <div className="grid grid-cols-4 gap-px bg-[var(--border-color)]">
        {TABS.map((item) => (
          <button
            key={item}
            onClick={() => setTab(item)}
            className="bg-black/20 px-2 py-2 text-[11px] uppercase tracking-[0.18em] text-slate-400 hover:bg-white/5"
            style={{ color: tab === item ? '#67e8f9' : undefined }}
          >
            {item}
          </button>
        ))}
      </div>

      <div className="flex-1 overflow-y-auto p-3">
        {tab === 'selection' && (
          <div className="space-y-3">
            <FlightPanel entity={selectedEntity} tracked={celebrity.trackedFlights} />
            <ShipPanel entity={selectedEntity} ships={fastData?.ships ?? []} />
            <SatellitePanel entity={selectedEntity} satellites={fastData?.satellites ?? []} />
          </div>
        )}
        {tab === 'news' && <NewsPanel news={slowData?.news ?? []} gdelt={slowData?.gdelt ?? []} />}
        {tab === 'markets' && <FinancialPanel stocks={slowData?.stocks ?? []} oil={slowData?.oil ?? []} celebrity={celebrity} />}
        {tab === 'earth' && <EarthObservationPanel earthquakes={slowData?.earthquakes ?? []} fires={slowData?.firms_fires ?? []} spaceWeather={slowData?.space_weather} weather={slowData?.weather} />}
        {tab === 'conflicts' && <ConflictPanel liveuamap={slowData?.liveuamap ?? []} frontlines={slowData?.frontlines} gdelt={slowData?.gdelt ?? []} />}
        {tab === 'infra' && <InfrastructurePanel outages={slowData?.internet_outages ?? []} kiwisdr={slowData?.kiwisdr ?? []} datacenters={slowData?.datacenters ?? []} cctv={slowData?.cctv ?? []} />}
        {tab === 'geocode' && <GeocodingPanel focusLocation={focusLocation} entity={selectedEntity} />}
      </div>
    </aside>
  )
}
