'use client'

import type { LayerVisibility } from '@/app/page'
import { AircraftIcon } from '@/components/map/icons/AircraftIcons'
import { SatelliteIcon } from '@/components/map/icons/SatelliteIcons'

const LABELS: Record<keyof LayerVisibility, string> = {
  commercial_flights: 'Commercial',
  private_flights: 'Private GA',
  private_jets: 'Private Jets',
  military_flights: 'Military',
  tracked_flights: 'PlaneAlert',
  uavs: 'UAVs',
  gps_jamming: 'GPS Jamming',
  ships: 'Ships',
  satellites: 'Satellites',
  earthquakes: 'Earthquakes',
  news: 'News',
  firms_fires: 'FIRMS',
  gdelt: 'GDELT',
  liveuamap: 'LiveUAmap',
  frontlines: 'Frontlines',
  internet_outages: 'Outages',
  kiwisdr: 'KiwiSDR',
  datacenters: 'Datacenters',
  cctv: 'CCTV',
  weather_radar: 'Weather',
  day_night: 'Day/Night',
}

export function MapLegend({ layers }: { layers: LayerVisibility }) {
  const active = Object.entries(layers).filter(([, enabled]) => enabled)
  return (
    <div className="rounded-2xl border border-white/10 bg-black/45 p-3 backdrop-blur-md">
      <div className="mb-2 text-[10px] uppercase tracking-[0.22em] text-cyan-300">Legend</div>
      <div className="mb-2 flex items-center gap-3 text-slate-300">
        <div className="flex items-center gap-1 text-xs"><AircraftIcon /> Aircraft</div>
        <div className="flex items-center gap-1 text-xs"><SatelliteIcon /> Satellite</div>
      </div>
      <div className="flex max-w-[16rem] flex-wrap gap-1.5">
        {active.map(([key]) => (
          <span key={key} className="rounded-full border border-white/10 bg-white/5 px-2 py-1 text-[11px] text-slate-300">
            {LABELS[key as keyof LayerVisibility]}
          </span>
        ))}
      </div>
    </div>
  )
}
