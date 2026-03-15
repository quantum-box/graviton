'use client'

import { useTranslation } from 'react-i18next'

interface StatusBarProps {
  coords: { lat: number; lng: number } | null
  zoom: number
  counts: Record<string, number>
  spaceWeather?: any
  statusLabel?: string
}

export function StatusBar({ coords, zoom, counts, spaceWeather, statusLabel = 'LIVE' }: StatusBarProps) {
  const flights = (counts.commercial_flights ?? 0) + (counts.private_flights ?? 0) + (counts.private_jets ?? 0) + (counts.military_flights ?? 0)
  const { t } = useTranslation()

  return (
    <div className="flex h-[var(--statusbar-height)] items-center justify-between gap-3 border-t border-[var(--panel-border)] bg-[var(--chrome-bg)] px-3 font-mono text-[11px] text-[var(--text-secondary)]">
      <div>
        {(coords
          ? t('status.latlng', { lat: coords.lat.toFixed(3), lng: coords.lng.toFixed(3) })
          : t('status.latlngEmpty'))}{' '}
        | {t('status.zoom', { zoom: zoom.toFixed(2) })} | {statusLabel}
      </div>
      <div className="hidden lg:block">
        {t('status.flights', { count: flights })} | {t('status.ships', { count: counts.ships ?? 0 })} | {t('status.sats', { count: counts.satellites ?? 0 })} |{' '}
        {t('status.kp', { value: spaceWeather?.kp_index ?? '-' })}
      </div>
    </div>
  )
}
