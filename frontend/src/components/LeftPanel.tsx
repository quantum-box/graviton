'use client'

import { LayerVisibility } from '@/app/page'
import { AlertTriangle, Flame, Newspaper, Plane, Radar, Radio, Satellite, Ship, ShieldAlert, Tv, Wifi, Zap } from 'lucide-react'
import { useTranslation } from 'react-i18next'

interface LeftPanelProps {
  layers: LayerVisibility
  counts: Record<string, number>
  onToggle: (key: keyof LayerVisibility) => void
  className?: string
}

const LAYERS: Array<{ key: keyof LayerVisibility; icon: any; color: string }> = [
  { key: 'commercial_flights', icon: Plane, color: '#38bdf8' },
  { key: 'private_flights', icon: Plane, color: '#818cf8' },
  { key: 'private_jets', icon: Plane, color: '#f472b6' },
  { key: 'military_flights', icon: ShieldAlert, color: '#ef4444' },
  { key: 'tracked_flights', icon: Radar, color: '#ec4899' },
  { key: 'uavs', icon: Radar, color: '#f59e0b' },
  { key: 'gps_jamming', icon: Zap, color: '#f97316' },
  { key: 'ships', icon: Ship, color: '#22c55e' },
  { key: 'satellites', icon: Satellite, color: '#fde047' },
  { key: 'earthquakes', icon: AlertTriangle, color: '#fb923c' },
  { key: 'firms_fires', icon: Flame, color: '#f97316' },
  { key: 'news', icon: Newspaper, color: '#a78bfa' },
  { key: 'gdelt', icon: AlertTriangle, color: '#f43f5e' },
  { key: 'liveuamap', icon: AlertTriangle, color: '#fb7185' },
  { key: 'frontlines', icon: AlertTriangle, color: '#facc15' },
  { key: 'internet_outages', icon: Wifi, color: '#14b8a6' },
  { key: 'kiwisdr', icon: Radio, color: '#67e8f9' },
  { key: 'datacenters', icon: Radar, color: '#94a3b8' },
  { key: 'cctv', icon: Tv, color: '#c084fc' },
  { key: 'weather_radar', icon: Radar, color: '#60a5fa' },
  { key: 'day_night', icon: Radar, color: '#fef08a' },
]

export function LeftPanel({ layers, counts, onToggle, className = '' }: LeftPanelProps) {
  const { t } = useTranslation()

  return (
    <aside className={`shrink-0 overflow-y-auto ${className}`}>
      <div className="space-y-2">
        <div className="mb-1 text-[11px] font-semibold uppercase tracking-[0.22em] text-[var(--text-muted)]">{t('layers.dataSources')}</div>
        <div className="space-y-2">
          {LAYERS.map(({ key, icon: Icon, color }) => {
            const active = layers[key]
            const count = counts[key] ?? 0
            return (
              <button
                key={key}
                onClick={() => onToggle(key)}
                className="flex w-full items-center gap-3 rounded-2xl border px-3 py-3 text-left transition"
                style={{
                  opacity: active ? 1 : 0.6,
                  borderColor: active ? 'rgba(94,234,212,0.22)' : 'rgba(255,255,255,0.08)',
                  background: active ? 'rgba(45,212,191,0.12)' : 'rgba(255,255,255,0.04)',
                }}
              >
                <div className="flex h-10 w-10 shrink-0 items-center justify-center rounded-2xl bg-black/25">
                  <Icon size={16} style={{ color }} />
                </div>
                <div className="min-w-0 flex-1">
                  <div className="truncate text-sm text-[var(--text-primary)]">{t(`layers.${key}`)}</div>
                  <div className="text-[11px] uppercase tracking-[0.18em] text-[var(--text-muted)]">{count} items</div>
                </div>
                <div
                  className={`flex h-6 w-11 shrink-0 items-center rounded-full p-1 transition ${active ? 'justify-end bg-teal-400/80' : 'justify-start bg-white/10'}`}
                  aria-hidden="true"
                >
                  <span className="h-4 w-4 rounded-full bg-white shadow-sm" />
                </div>
              </button>
            )
          })}
        </div>
      </div>
    </aside>
  )
}
