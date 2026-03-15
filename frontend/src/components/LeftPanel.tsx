'use client'

import { LayerVisibility } from '@/app/page'
import { AlertTriangle, Flame, Newspaper, Plane, Radar, Radio, Satellite, Ship, ShieldAlert, Tv, Wifi, Zap } from 'lucide-react'

interface LeftPanelProps {
  layers: LayerVisibility
  counts: Record<string, number>
  onToggle: (key: keyof LayerVisibility) => void
}

const LAYERS: Array<{ key: keyof LayerVisibility; label: string; icon: any; color: string }> = [
  { key: 'commercial_flights', label: 'Commercial', icon: Plane, color: '#38bdf8' },
  { key: 'private_flights', label: 'Private GA', icon: Plane, color: '#818cf8' },
  { key: 'private_jets', label: 'Private Jets', icon: Plane, color: '#f472b6' },
  { key: 'military_flights', label: 'Military Flights', icon: ShieldAlert, color: '#ef4444' },
  { key: 'tracked_flights', label: 'PlaneAlert', icon: Radar, color: '#ec4899' },
  { key: 'uavs', label: 'UAVs', icon: Radar, color: '#f59e0b' },
  { key: 'gps_jamming', label: 'GPS Jamming', icon: Zap, color: '#f97316' },
  { key: 'ships', label: 'AIS / Carriers', icon: Ship, color: '#22c55e' },
  { key: 'satellites', label: 'SGP4 / Sentinel', icon: Satellite, color: '#fde047' },
  { key: 'earthquakes', label: 'Earthquakes', icon: AlertTriangle, color: '#fb923c' },
  { key: 'firms_fires', label: 'FIRMS Fires', icon: Flame, color: '#f97316' },
  { key: 'news', label: 'RSS / Risk', icon: Newspaper, color: '#a78bfa' },
  { key: 'gdelt', label: 'GDELT', icon: AlertTriangle, color: '#f43f5e' },
  { key: 'liveuamap', label: 'LiveUAmap', icon: AlertTriangle, color: '#fb7185' },
  { key: 'frontlines', label: 'Frontlines', icon: AlertTriangle, color: '#facc15' },
  { key: 'internet_outages', label: 'Internet Outages', icon: Wifi, color: '#14b8a6' },
  { key: 'kiwisdr', label: 'KiwiSDR', icon: Radio, color: '#67e8f9' },
  { key: 'datacenters', label: 'Data Centers', icon: Radar, color: '#94a3b8' },
  { key: 'cctv', label: 'CCTV', icon: Tv, color: '#c084fc' },
  { key: 'weather_radar', label: 'Weather Radar', icon: Radar, color: '#60a5fa' },
  { key: 'day_night', label: 'Day/Night', icon: Radar, color: '#fef08a' },
]

export function LeftPanel({ layers, counts, onToggle }: LeftPanelProps) {
  return (
    <aside className="w-[272px] shrink-0 overflow-y-auto border-r border-[var(--border-color)] bg-[linear-gradient(180deg,#0a1320,#0c1829)]">
      <div className="p-3">
        <div className="mb-3 text-[11px] font-semibold uppercase tracking-[0.22em] text-slate-500">Data Sources</div>
        <div className="space-y-1.5">
          {LAYERS.map(({ key, label, icon: Icon, color }) => {
            const active = layers[key]
            const count = counts[key] ?? 0
            return (
              <button
                key={key}
                onClick={() => onToggle(key)}
                className="flex w-full items-center gap-2 rounded-lg px-2 py-2 text-left transition hover:bg-white/5"
                style={{ opacity: active ? 1 : 0.45, background: active ? 'rgba(255,255,255,0.04)' : undefined }}
              >
                <Icon size={15} style={{ color }} />
                <span className="flex-1 text-sm">{label}</span>
                <span className="rounded bg-black/20 px-1.5 py-0.5 font-mono text-[11px] text-slate-300">{count}</span>
              </button>
            )
          })}
        </div>
      </div>
    </aside>
  )
}
