'use client'

import { CompactList, PanelCard } from './common'

export function EarthObservationPanel({ earthquakes, fires, spaceWeather, weather }: { earthquakes: any[]; fires: any[]; spaceWeather: any; weather: any }) {
  return (
    <div className="space-y-3">
      <PanelCard title="Earthquakes" subtitle="USGS">
        <CompactList items={earthquakes.map((q) => ({ title: `${q.place}`, meta: `M${q.magnitude ?? q.mag}` }))} />
      </PanelCard>
      <PanelCard title="Fires" subtitle="NASA FIRMS">
        <CompactList items={fires.map((fire) => ({ title: `${fire.acq_date || ''} ${fire.acq_time || ''}`.trim() || 'Hotspot', meta: `FRP ${fire.frp} | ${fire.confidence}` }))} />
      </PanelCard>
      <PanelCard title="Space Weather" subtitle="NOAA SWPC">
        <div className="text-sm">Kp {spaceWeather?.kp_index ?? '-'} | {spaceWeather?.status || spaceWeather?.kp_text || 'unknown'}</div>
      </PanelCard>
      <PanelCard title="Weather" subtitle="RainViewer">
        <div className="text-sm text-slate-400">{weather?.radar_tile_path ? 'Radar overlay available.' : 'No radar tiles loaded.'}</div>
      </PanelCard>
    </div>
  )
}
