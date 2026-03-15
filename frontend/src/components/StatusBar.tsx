'use client'

interface StatusBarProps {
  coords: { lat: number; lng: number } | null
  zoom: number
  counts: Record<string, number>
  spaceWeather?: any
}

export function StatusBar({ coords, zoom, counts, spaceWeather }: StatusBarProps) {
  const flights = (counts.commercial_flights ?? 0) + (counts.private_flights ?? 0) + (counts.private_jets ?? 0) + (counts.military_flights ?? 0)

  return (
    <div className="flex h-[var(--statusbar-height)] items-center justify-between border-t border-[var(--border-color)] bg-black/30 px-3 font-mono text-[11px] text-slate-400">
      <div>
        {coords ? `LAT ${coords.lat.toFixed(3)} LNG ${coords.lng.toFixed(3)}` : 'LAT -- LNG --'} | Z {zoom.toFixed(2)}
      </div>
      <div className="hidden md:block">
        FLIGHTS {flights} | SHIPS {counts.ships ?? 0} | SATS {counts.satellites ?? 0} | KP {spaceWeather?.kp_index ?? '-'}
      </div>
    </div>
  )
}
