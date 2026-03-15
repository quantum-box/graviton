'use client'

import { useState } from 'react'

export function C2Panel({
  token,
  state,
  selectedEntity,
  onRefresh,
  position,
  onMove,
}: {
  token: string | null
  state: { watchlist: any[]; missions: any[]; alerts: any[] } | null
  selectedEntity: any
  onRefresh: () => void
  position: { x: number; y: number }
  onMove: (position: { x: number; y: number }) => void
}) {
  const [missionTitle, setMissionTitle] = useState('')
  const [note, setNote] = useState('')

  async function createWatchlist() {
    if (!token || !selectedEntity) return
    await fetch('/api/c2/watchlist', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json', Authorization: `Bearer ${token}` },
      body: JSON.stringify({
        target_id: selectedEntity.icao24 ?? selectedEntity.mmsi ?? selectedEntity.id ?? 'unknown',
        target_type: selectedEntity.type ?? selectedEntity.sourceType ?? 'track',
        note,
      }),
    })
    onRefresh()
  }

  async function createMission() {
    if (!token || !missionTitle) return
    await fetch('/api/c2/missions', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json', Authorization: `Bearer ${token}` },
      body: JSON.stringify({
        title: missionTitle,
        status: 'planned',
        target: selectedEntity ?? null,
      }),
    })
    setMissionTitle('')
    onRefresh()
  }

  async function shareSelection() {
    if (!token || !selectedEntity) return
    await fetch('/api/team/shared', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json', Authorization: `Bearer ${token}` },
      body: JSON.stringify({
        kind: 'marker',
        title: selectedEntity.callsign ?? selectedEntity.name ?? 'Shared marker',
        payload: {
          lat: selectedEntity.lat,
          lng: selectedEntity.lng,
          label: selectedEntity.callsign ?? selectedEntity.name ?? 'Shared marker',
        },
      }),
    })
  }

  return (
    <div className="pointer-events-auto absolute z-30 w-[340px] rounded-[28px] border border-white/10 bg-[rgba(7,14,24,0.92)] p-4 shadow-[0_24px_72px_rgba(0,0,0,0.42)] backdrop-blur-xl" style={{ left: position.x, top: position.y }}>
      <div className="mb-3 flex items-center justify-between">
        <div>
          <div className="text-[11px] uppercase tracking-[0.22em] text-slate-400">C2 Interface</div>
          <div className="mt-1 text-sm text-white">Watchlist, alert rules, missions</div>
        </div>
        <a href="/api/docs" target="_blank" className="rounded-full border border-white/10 px-3 py-1 text-xs text-slate-300">
          API Docs
        </a>
      </div>
      <div className="mb-3 flex gap-2">
        {[
          { label: 'TL', x: 20, y: 120 },
          { label: 'TR', x: 980, y: 120 },
          { label: 'BL', x: 20, y: 420 },
          { label: 'BR', x: 980, y: 420 },
        ].map((preset) => (
          <button key={preset.label} onClick={() => onMove({ x: preset.x, y: preset.y })} className="rounded-full border border-white/10 px-2 py-1 text-[10px] text-slate-300">
            {preset.label}
          </button>
        ))}
      </div>
      <div className="space-y-3">
        <div className="rounded-2xl border border-white/8 bg-white/5 p-3">
          <div className="mb-2 text-[11px] uppercase tracking-[0.18em] text-slate-400">Watch Target</div>
          <div className="text-sm text-white">{selectedEntity?.callsign ?? selectedEntity?.name ?? selectedEntity?.icao24 ?? 'No selection'}</div>
          <input value={note} onChange={(e) => setNote(e.target.value)} placeholder="Note / alert rule context" className="mt-2 w-full rounded-2xl border border-white/10 bg-black/20 px-3 py-2 text-sm text-white outline-none" />
          <button onClick={() => void createWatchlist()} disabled={!token || !selectedEntity} className="mt-2 w-full rounded-2xl bg-teal-400/20 px-4 py-2 text-sm text-teal-100 disabled:opacity-50">
            Add to watchlist
          </button>
        </div>
        <div className="rounded-2xl border border-white/8 bg-white/5 p-3">
          <div className="mb-2 text-[11px] uppercase tracking-[0.18em] text-slate-400">Mission</div>
          <input value={missionTitle} onChange={(e) => setMissionTitle(e.target.value)} placeholder="Mission title" className="w-full rounded-2xl border border-white/10 bg-black/20 px-3 py-2 text-sm text-white outline-none" />
          <button onClick={() => void createMission()} disabled={!token} className="mt-2 w-full rounded-2xl bg-fuchsia-400/20 px-4 py-2 text-sm text-fuchsia-100 disabled:opacity-50">
            Create mission
          </button>
        </div>
        <div className="grid grid-cols-3 gap-2 text-center">
          <div className="rounded-2xl bg-white/5 p-2">
            <div className="text-[10px] uppercase tracking-[0.18em] text-slate-400">Alerts</div>
            <div className="mt-1 text-lg text-white">{state?.alerts?.length ?? 0}</div>
          </div>
          <div className="rounded-2xl bg-white/5 p-2">
            <div className="text-[10px] uppercase tracking-[0.18em] text-slate-400">Watchlist</div>
            <div className="mt-1 text-lg text-white">{state?.watchlist?.length ?? 0}</div>
          </div>
          <div className="rounded-2xl bg-white/5 p-2">
            <div className="text-[10px] uppercase tracking-[0.18em] text-slate-400">Missions</div>
            <div className="mt-1 text-lg text-white">{state?.missions?.length ?? 0}</div>
          </div>
        </div>
        <button onClick={() => void shareSelection()} disabled={!token || !selectedEntity} className="w-full rounded-2xl bg-emerald-400/20 px-4 py-2 text-sm text-emerald-100 disabled:opacity-50">
          Share selected marker
        </button>
        <a href="/api/report/summary" target="_blank" className="block rounded-2xl border border-white/10 px-4 py-2 text-center text-sm text-slate-300">
          Open markdown report
        </a>
      </div>
    </div>
  )
}
