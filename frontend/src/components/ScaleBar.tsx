'use client'

import { useMemo } from 'react'

const DISTANCE_STEPS = [50, 100, 200, 500, 1000, 2000, 5000, 10000] as const
const MAX_WIDTH_PX = 120

function formatDistance(km: number) {
  if (km < 1) return `${Math.round(km * 1000)} m`
  if (km >= 1000) return `${(km / 1000).toFixed(km >= 5000 ? 0 : 1)}k km`
  return `${Math.round(km)} km`
}

export function ScaleBar({ zoom, latitude }: { zoom: number; latitude: number }) {
  const scale = useMemo(() => {
    const metersPerPixel = (156543.03392 * Math.cos((latitude * Math.PI) / 180)) / 2 ** zoom
    const pickedStep =
      DISTANCE_STEPS.find((stepKm) => (stepKm * 1000) / metersPerPixel <= MAX_WIDTH_PX) ?? DISTANCE_STEPS[0]
    const width = Math.max(24, Math.min(MAX_WIDTH_PX, (pickedStep * 1000) / metersPerPixel))
    return { label: formatDistance(pickedStep), width }
  }, [latitude, zoom])

  return (
    <div className="pointer-events-none absolute bottom-4 left-4 z-20 rounded-2xl border border-white/10 bg-[#07111b]/85 px-3 py-2 text-[11px] text-slate-200 shadow-xl backdrop-blur-sm">
      <div className="mb-1 uppercase tracking-[0.18em] text-slate-500">Scale</div>
      <div className="flex items-end gap-2">
        <div className="relative h-3">
          <div className="absolute bottom-0 left-0 h-2 border-l border-r border-t border-cyan-300" style={{ width: `${scale.width}px` }} />
        </div>
        <span className="font-mono text-cyan-300">{scale.label}</span>
      </div>
    </div>
  )
}

