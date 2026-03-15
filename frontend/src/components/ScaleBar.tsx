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
    <div className="pointer-events-none absolute bottom-20 right-4 z-20 rounded-lg border border-white/8 bg-black/38 px-2 py-1.5 text-[10px] text-slate-300 shadow-[0_8px_24px_rgba(0,0,0,0.22)] backdrop-blur-[2px]">
      <div className="mb-1 uppercase tracking-[0.14em] text-[9px] text-slate-500">Scale</div>
      <div className="flex items-end gap-1.5">
        <div className="relative h-2.5">
          <div className="absolute bottom-0 left-0 h-1.5 border-l border-r border-t border-cyan-300/85" style={{ width: `${scale.width}px` }} />
        </div>
        <span className="font-mono text-[10px] text-cyan-300/90">{scale.label}</span>
      </div>
    </div>
  )
}
