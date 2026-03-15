'use client'

import Map from 'react-map-gl/maplibre'
import 'maplibre-gl/dist/maplibre-gl.css'
import { useTranslation } from 'react-i18next'

export default function NavigatorMiniMap({
  center,
  focusLocation,
}: {
  center: { lat: number; lng: number; zoom: number }
  focusLocation: { lat: number; lng: number; label?: string } | null
}) {
  const { t } = useTranslation()

  return (
    <div className="space-y-3">
      <div className="relative h-40 overflow-hidden rounded-[10px] border border-[var(--panel-border)]">
        <Map
          initialViewState={{ longitude: center.lng, latitude: center.lat, zoom: Math.max(0.8, center.zoom - 2.4) }}
          longitude={center.lng}
          latitude={center.lat}
          zoom={Math.max(0.8, center.zoom - 2.4)}
          mapStyle="https://basemaps.cartocdn.com/gl/dark-matter-gl-style/style.json"
          attributionControl={false}
          dragPan={false}
          scrollZoom={false}
          doubleClickZoom={false}
          touchZoomRotate={false}
          keyboard={false}
        />
        <div className="pointer-events-none absolute inset-0 flex items-center justify-center">
          <div className="h-10 w-10 rounded-full border border-[rgba(255,153,0,0.6)]" />
        </div>
        <div className="pointer-events-none absolute inset-x-0 top-1/2 h-px -translate-y-1/2 bg-[rgba(255,255,255,0.16)]" />
        <div className="pointer-events-none absolute inset-y-0 left-1/2 w-px -translate-x-1/2 bg-[rgba(255,255,255,0.16)]" />
      </div>

      <div className="grid grid-cols-2 gap-2 text-xs">
        <div className="rounded-[8px] border border-[var(--panel-border)] bg-[#191919] px-3 py-2">
          <div className="text-[10px] uppercase tracking-[0.2em] text-[var(--text-muted)]">{t('shell.center')}</div>
          <div className="mt-1 font-mono text-[var(--text-primary)]">{center.lat.toFixed(2)}, {center.lng.toFixed(2)}</div>
        </div>
        <div className="rounded-[8px] border border-[var(--panel-border)] bg-[#191919] px-3 py-2">
          <div className="text-[10px] uppercase tracking-[0.2em] text-[var(--text-muted)]">{t('shell.zoom')}</div>
          <div className="mt-1 font-mono text-[var(--text-primary)]">{center.zoom.toFixed(2)}</div>
        </div>
      </div>

      <div className="rounded-[8px] border border-[var(--panel-border)] bg-[#191919] px-3 py-2 text-xs text-[var(--text-secondary)]">
        <div className="text-[10px] uppercase tracking-[0.2em] text-[var(--text-muted)]">{t('shell.pinnedLocation')}</div>
        <div className="mt-1 text-[var(--text-primary)]">{focusLocation?.label || t('shell.none')}</div>
      </div>
    </div>
  )
}
