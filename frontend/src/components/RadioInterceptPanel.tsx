'use client'

import { useEffect, useState } from 'react'
import type { FocusLocation } from '@/types/dashboard'

export function RadioInterceptPanel({ location }: { location: FocusLocation | null }) {
  const [topFeeds, setTopFeeds] = useState<any[]>([])
  const [nearestFeeds, setNearestFeeds] = useState<any[]>([])

  useEffect(() => {
    fetch('/api/radio/top')
      .then((resp) => resp.json())
      .then((data) => setTopFeeds(Array.isArray(data) ? data : []))
      .catch(() => setTopFeeds([]))
  }, [])

  useEffect(() => {
    if (!location) return
    fetch(`/api/radio/nearest?lat=${location.lat}&lng=${location.lng}`)
      .then((resp) => resp.json())
      .then((data) => setNearestFeeds(Array.isArray(data) ? data : []))
      .catch(() => setNearestFeeds([]))
  }, [location?.lat, location?.lng])

  return (
    <section className="rounded-2xl border border-white/10 bg-white/[0.03] p-3">
      <div className="mb-2 text-[11px] uppercase tracking-[0.2em] text-cyan-300">Radio Intercept</div>
      <div className="space-y-2">
        {(location ? nearestFeeds : topFeeds).slice(0, 6).map((feed: any, index) => (
          <a
            key={`${feed.id ?? index}`}
            href={feed.stream_url || '#'}
            target="_blank"
            className="block rounded-xl bg-black/20 p-2 text-sm text-slate-100 hover:text-cyan-300"
          >
            <div>{feed.name || feed.shortName || 'Scanner feed'}</div>
            <div className="text-xs text-slate-500">{feed.location || feed.countyName || feed.category || 'OpenMHz / Broadcastify'}</div>
          </a>
        ))}
      </div>
    </section>
  )
}

