'use client'

import { LayerVisibility } from '@/app/page'
import {
  Plane, Ship, Satellite, AlertTriangle, Newspaper,
  Flame, Globe2, Wifi, Radio, CloudRain, Sun
} from 'lucide-react'

interface LeftPanelProps {
  layers: LayerVisibility
  counts: Record<string, number>
  onToggle: (key: keyof LayerVisibility) => void
}

const LAYER_CONFIG: {
  key: keyof LayerVisibility
  label: string
  icon: any
  color: string
  tier: 'fast' | 'slow'
}[] = [
  { key: 'commercial_flights', label: 'Commercial Flights', icon: Plane, color: '#00a8ff', tier: 'fast' },
  { key: 'military_flights', label: 'Military Aircraft', icon: Plane, color: '#ff5252', tier: 'fast' },
  { key: 'ships', label: 'Ships & Vessels', icon: Ship, color: '#00e676', tier: 'fast' },
  { key: 'satellites', label: 'Satellites', icon: Satellite, color: '#ffd740', tier: 'fast' },
  { key: 'earthquakes', label: 'Earthquakes', icon: AlertTriangle, color: '#ff9100', tier: 'slow' },
  { key: 'news', label: 'News Events', icon: Newspaper, color: '#7c4dff', tier: 'slow' },
  { key: 'firms_fires', label: 'FIRMS Fires', icon: Flame, color: '#ff6d00', tier: 'slow' },
  { key: 'gdelt', label: 'GDELT Conflicts', icon: Globe2, color: '#ff1744', tier: 'slow' },
  { key: 'internet_outages', label: 'Internet Outages', icon: Wifi, color: '#00bfa5', tier: 'slow' },
  { key: 'kiwisdr', label: 'KiwiSDR Receivers', icon: Radio, color: '#64ffda', tier: 'slow' },
  { key: 'weather_radar', label: 'Weather Radar', icon: CloudRain, color: '#448aff', tier: 'slow' },
  { key: 'day_night', label: 'Day/Night Cycle', icon: Sun, color: '#fff176', tier: 'slow' },
]

export function LeftPanel({ layers, counts, onToggle }: LeftPanelProps) {
  return (
    <div
      className="shrink-0 overflow-y-auto"
      style={{
        width: 260,
        background: 'var(--bg-secondary)',
        borderRight: '1px solid var(--border-color)',
      }}
    >
      <div className="p-3">
        <div className="text-xs font-semibold uppercase tracking-wider mb-3" style={{ color: 'var(--text-secondary)' }}>
          Data Layers
        </div>

        <div className="space-y-1">
          {LAYER_CONFIG.map(({ key, label, icon: Icon, color, tier }) => {
            const active = layers[key]
            const count = counts[key] ?? 0
            return (
              <button
                key={key}
                onClick={() => onToggle(key)}
                className="w-full flex items-center gap-2 px-2 py-1.5 rounded text-left transition-all group"
                style={{
                  background: active ? 'rgba(255,255,255,0.05)' : 'transparent',
                  opacity: active ? 1 : 0.5,
                }}
              >
                <div
                  className="w-5 h-5 flex items-center justify-center rounded"
                  style={{ color: active ? color : 'var(--text-secondary)' }}
                >
                  <Icon size={14} />
                </div>
                <span className="flex-1 text-xs truncate" style={{ color: active ? 'var(--text-primary)' : 'var(--text-secondary)' }}>
                  {label}
                </span>
                {count > 0 && active && (
                  <span
                    className="text-xs font-mono px-1.5 py-0.5 rounded"
                    style={{ background: 'rgba(255,255,255,0.08)', color }}
                  >
                    {count > 999 ? `${(count / 1000).toFixed(1)}k` : count}
                  </span>
                )}
                <div
                  className="w-2 h-2 rounded-full"
                  style={{
                    background: active ? color : 'var(--border-color)',
                    boxShadow: active ? `0 0 4px ${color}` : 'none',
                  }}
                />
              </button>
            )
          })}
        </div>
      </div>

      <div className="p-3 border-t" style={{ borderColor: 'var(--border-color)' }}>
        <div className="text-xs font-semibold uppercase tracking-wider mb-2" style={{ color: 'var(--text-secondary)' }}>
          Refresh Rates
        </div>
        <div className="space-y-1 text-xs" style={{ color: 'var(--text-secondary)' }}>
          <div className="flex justify-between">
            <span>Fast tier (flights, ships)</span>
            <span className="font-mono text-green-400">15s</span>
          </div>
          <div className="flex justify-between">
            <span>Slow tier (news, quakes)</span>
            <span className="font-mono text-yellow-400">120s</span>
          </div>
        </div>
      </div>
    </div>
  )
}
