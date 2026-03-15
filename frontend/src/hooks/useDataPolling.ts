'use client'

import { useCallback, useEffect, useRef, useState } from 'react'
import type { FastData, SlowData } from '@/types/dashboard'

const EMPTY_FAST: FastData = {
  commercial_flights: [],
  private_flights: [],
  private_jets: [],
  military_flights: [],
  tracked_flights: [],
  uavs: [],
  gps_jamming: [],
  ships: [],
  satellites: [],
  fused_objects: [],
  simulation_markers: [],
  alerts: [],
}

const EMPTY_SLOW: SlowData = {
  earthquakes: [],
  news: [],
  stocks: [],
  oil: [],
  firms_fires: [],
  gdelt: [],
  liveuamap: [],
  frontlines: { type: 'FeatureCollection', features: [] },
  internet_outages: [],
  kiwisdr: [],
  datacenters: [],
  cctv: [],
  shared_objects: [],
}

export function useDataPolling() {
  const [fastData, setFastData] = useState<FastData | null>(null)
  const [slowData, setSlowData] = useState<SlowData | null>(null)
  const [isLoading, setIsLoading] = useState(true)
  const fastEtag = useRef('')
  const slowEtag = useRef('')

  const fetchFast = useCallback(async () => {
    const headers: HeadersInit = {}
    if (fastEtag.current) headers['If-None-Match'] = fastEtag.current
    const resp = await fetch('/api/live-data/fast', { headers })
    if (resp.status === 304) return
    if (!resp.ok) throw new Error('fast data fetch failed')
    fastEtag.current = resp.headers.get('etag') ?? fastEtag.current
    setFastData({ ...EMPTY_FAST, ...(await resp.json()) })
  }, [])

  const fetchSlow = useCallback(async () => {
    const headers: HeadersInit = {}
    if (slowEtag.current) headers['If-None-Match'] = slowEtag.current
    const resp = await fetch('/api/live-data/slow', { headers })
    if (resp.status === 304) return
    if (!resp.ok) throw new Error('slow data fetch failed')
    slowEtag.current = resp.headers.get('etag') ?? slowEtag.current
    setSlowData({ ...EMPTY_SLOW, ...(await resp.json()) })
  }, [])

  useEffect(() => {
    let active = true
    const run = async () => {
      try {
        await Promise.all([fetchFast(), fetchSlow()])
      } catch (error) {
        console.warn(error)
      } finally {
        if (active) setIsLoading(false)
      }
    }
    run()
    const fastTimer = window.setInterval(() => void fetchFast().catch(console.warn), 15000)
    const slowTimer = window.setInterval(() => void fetchSlow().catch(console.warn), 120000)
    return () => {
      active = false
      window.clearInterval(fastTimer)
      window.clearInterval(slowTimer)
    }
  }, [fetchFast, fetchSlow])

  return { fastData, slowData, isLoading }
}
