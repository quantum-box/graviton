'use client'

import { Search, MapPinned } from 'lucide-react'
import { useState } from 'react'
import type { FocusLocation } from '@/types/dashboard'

export function FindLocateBar({ onSelect }: { onSelect: (location: FocusLocation) => void }) {
  const [query, setQuery] = useState('')
  const [results, setResults] = useState<FocusLocation[]>([])
  const [loading, setLoading] = useState(false)

  async function handleSearch(value: string) {
    setQuery(value)
    const coordMatch = value.trim().match(/^([+-]?\d+\.?\d*)[,\s]+([+-]?\d+\.?\d*)$/)
    if (coordMatch) {
      setResults([{ lat: Number(coordMatch[1]), lng: Number(coordMatch[2]), label: 'Coordinates' }])
      return
    }
    if (value.trim().length < 2) {
      setResults([])
      return
    }
    setLoading(true)
    try {
      const resp = await fetch(`/api/geocode/search?q=${encodeURIComponent(value.trim())}`)
      const data = await resp.json()
      setResults(
        (Array.isArray(data) ? data : []).slice(0, 6).map((item: any) => ({
          lat: Number(item.lat),
          lng: Number(item.lon),
          label: item.display_name,
        })),
      )
    } finally {
      setLoading(false)
    }
  }

  return (
    <div className="relative w-full max-w-xl">
      <div className="flex items-center gap-2 rounded-2xl border border-cyan-900/60 bg-black/35 px-3 py-2 backdrop-blur-md">
        <Search size={15} className="text-cyan-300" />
        <input
          value={query}
          onChange={(e) => handleSearch(e.target.value)}
          placeholder="Search coordinates, cities, ports, airbases, conflict zones"
          className="w-full bg-transparent text-sm text-slate-100 outline-none placeholder:text-slate-500"
        />
        {loading && <div className="h-3 w-3 animate-spin rounded-full border border-cyan-300 border-t-transparent" />}
      </div>
      {results.length > 0 && (
        <div className="absolute left-0 right-0 top-[calc(100%+0.35rem)] z-30 overflow-hidden rounded-2xl border border-white/10 bg-[#08101b]/95 shadow-2xl">
          {results.map((item, index) => (
            <button
              key={`${item.label}-${index}`}
              onClick={() => {
                onSelect(item)
                setResults([])
              }}
              className="flex w-full items-start gap-2 border-b border-white/5 px-3 py-2 text-left last:border-b-0 hover:bg-white/5"
            >
              <MapPinned size={14} className="mt-0.5 shrink-0 text-cyan-300" />
              <span className="text-xs text-slate-300">{item.label || `${item.lat}, ${item.lng}`}</span>
            </button>
          ))}
        </div>
      )}
    </div>
  )
}

