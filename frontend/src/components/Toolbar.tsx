'use client'

import { useEffect, useState } from 'react'
import { Globe, Loader2, PanelLeftClose, PanelLeftOpen, PanelRightClose, PanelRightOpen, Search } from 'lucide-react'

interface ToolbarProps {
  leftOpen: boolean
  rightOpen: boolean
  onToggleLeft: () => void
  onToggleRight: () => void
  onSearchSelect: (result: { lat: number; lng: number; label?: string } | null) => void
  isLoading: boolean
}

export function Toolbar({ leftOpen, rightOpen, onToggleLeft, onToggleRight, onSearchSelect, isLoading }: ToolbarProps) {
  const [time, setTime] = useState('')
  const [query, setQuery] = useState('')
  const [searching, setSearching] = useState(false)

  useEffect(() => {
    const tick = () => setTime(new Date().toISOString().replace('T', ' ').slice(0, 19) + ' UTC')
    tick()
    const id = setInterval(tick, 1000)
    return () => clearInterval(id)
  }, [])

  async function handleSearch(e: React.FormEvent) {
    e.preventDefault()
    if (!query.trim()) return
    setSearching(true)
    try {
      const resp = await fetch(`/api/geocode/search?q=${encodeURIComponent(query.trim())}`)
      const data = await resp.json()
      const hit = Array.isArray(data) ? data[0] : null
      if (hit) {
        onSearchSelect({
          lat: Number(hit.lat),
          lng: Number(hit.lon),
          label: hit.display_name,
        })
      }
    } finally {
      setSearching(false)
    }
  }

  return (
    <div className="flex h-[var(--toolbar-height)] items-center justify-between gap-3 border-b border-[var(--border-color)] bg-[linear-gradient(90deg,#09111f,#10233d_40%,#0f1727)] px-3">
      <div className="flex items-center gap-2">
        <button onClick={onToggleLeft} className="rounded border border-white/10 p-1 text-slate-300 hover:bg-white/10">
          {leftOpen ? <PanelLeftClose size={15} /> : <PanelLeftOpen size={15} />}
        </button>
        <div className="flex items-center gap-2 font-semibold tracking-[0.3em] text-cyan-300">
          <Globe size={15} />
          <span>GRAVITON</span>
          {(isLoading || searching) && <Loader2 size={14} className="animate-spin" />}
        </div>
      </div>

      <form onSubmit={handleSearch} className="flex min-w-0 max-w-xl flex-1 items-center gap-2">
        <div className="flex flex-1 items-center gap-2 rounded-full border border-cyan-900/60 bg-black/20 px-3 py-1">
          <Search size={14} className="text-cyan-300" />
          <input
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder="Geocode a city, base, port, or conflict zone"
            className="w-full bg-transparent text-sm outline-none placeholder:text-slate-500"
          />
        </div>
      </form>

      <div className="flex items-center gap-3">
        <div className="hidden font-mono text-xs text-slate-400 md:block">{time}</div>
        <button onClick={onToggleRight} className="rounded border border-white/10 p-1 text-slate-300 hover:bg-white/10">
          {rightOpen ? <PanelRightClose size={15} /> : <PanelRightOpen size={15} />}
        </button>
      </div>
    </div>
  )
}
