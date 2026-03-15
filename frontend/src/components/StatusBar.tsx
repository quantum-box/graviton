'use client'

import { useTranslation } from 'react-i18next'

interface StatusBarProps {
  coords: { lat: number; lng: number } | null
  zoom: number
  counts: Record<string, number>
  spaceWeather?: any
  statusLabel?: string
}

export function StatusBar({ coords, zoom, statusLabel = 'LIVE' }: StatusBarProps) {
  const { t } = useTranslation()

  return (
    <div className="pointer-events-auto flex h-[var(--statusbar-height)] items-center justify-between gap-3 rounded-full border border-white/10 bg-[rgba(7,14,24,0.76)] px-3 font-mono text-[11px] text-slate-300 shadow-[0_12px_36px_rgba(0,0,0,0.28)] backdrop-blur-xl">
      <div className="truncate">
        {coords ? t('status.latlng', { lat: coords.lat.toFixed(3), lng: coords.lng.toFixed(3) }) : t('status.latlngEmpty')}
      </div>
      <div className="shrink-0">{t('status.zoom', { zoom: zoom.toFixed(2) })}</div>
      {statusLabel && <div className="hidden truncate text-slate-400 md:block">{statusLabel}</div>}
    </div>
  )
}
