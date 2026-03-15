import { useState, useEffect, useRef, useCallback } from 'react'

interface FastData {
  last_updated: string
  commercial_flights: any[]
  military_flights: any[]
  ships: any[]
  satellites: any[]
}

interface SlowData {
  last_updated: string
  earthquakes: any[]
  news: any[]
  stocks: any[]
  oil: any[]
  firms_fires: any[]
  gdelt: any[]
  space_weather: any
  weather: any
  internet_outages: any[]
  kiwisdr: any[]
}

export function useDataPolling() {
  const [fastData, setFastData] = useState<FastData | null>(null)
  const [slowData, setSlowData] = useState<SlowData | null>(null)
  const [isLoading, setIsLoading] = useState(true)
  const fastEtag = useRef<string>('')
  const slowEtag = useRef<string>('')
  const fastInterval = useRef(3000) // Start fast, slow down
  const slowInterval = useRef(5000)

  const fetchFast = useCallback(async () => {
    try {
      const headers: HeadersInit = {}
      if (fastEtag.current) {
        headers['If-None-Match'] = fastEtag.current
      }
      const resp = await fetch('/api/live-data/fast', { headers })
      if (resp.status === 304) return
      if (resp.ok) {
        const data = await resp.json()
        setFastData(data)
        const etag = resp.headers.get('etag')
        if (etag) fastEtag.current = etag
        // Slow down to steady-state after first success
        fastInterval.current = 15000
      }
    } catch (e) {
      console.warn('Fast data fetch failed:', e)
    } finally {
      setIsLoading(false)
    }
  }, [])

  const fetchSlow = useCallback(async () => {
    try {
      const headers: HeadersInit = {}
      if (slowEtag.current) {
        headers['If-None-Match'] = slowEtag.current
      }
      const resp = await fetch('/api/live-data/slow', { headers })
      if (resp.status === 304) return
      if (resp.ok) {
        const data = await resp.json()
        setSlowData(data)
        const etag = resp.headers.get('etag')
        if (etag) slowEtag.current = etag
        slowInterval.current = 120000
      }
    } catch (e) {
      console.warn('Slow data fetch failed:', e)
    }
  }, [])

  useEffect(() => {
    fetchFast()
    fetchSlow()

    const fastTimer = setInterval(() => fetchFast(), fastInterval.current)
    const slowTimer = setInterval(() => fetchSlow(), slowInterval.current)

    // Upgrade intervals after initial burst
    const upgradeTimer = setTimeout(() => {
      clearInterval(fastTimer)
      clearInterval(slowTimer)
      setInterval(() => fetchFast(), 15000)
      setInterval(() => fetchSlow(), 120000)
    }, 30000)

    return () => {
      clearInterval(fastTimer)
      clearInterval(slowTimer)
      clearTimeout(upgradeTimer)
    }
  }, [fetchFast, fetchSlow])

  return { fastData, slowData, isLoading }
}
