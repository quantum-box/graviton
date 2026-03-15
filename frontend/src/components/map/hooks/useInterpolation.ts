'use client'

import { useEffect, useMemo, useRef, useState } from 'react'
import type { BaseEntity } from '@/types/dashboard'

type EntityMap = Record<string, BaseEntity>

function entityKey(item: BaseEntity, fallback: string) {
  return String(item.id ?? item.icao24 ?? item.mmsi ?? item.norad_id ?? item.name ?? fallback)
}

export function useInterpolation(items: BaseEntity[], duration = 1200) {
  const previous = useRef<EntityMap>({})
  const [progress, setProgress] = useState(1)

  useEffect(() => {
    let frame = 0
    const started = performance.now()
    const tick = (now: number) => {
      const next = Math.min(1, (now - started) / duration)
      setProgress(next)
      if (next < 1) frame = requestAnimationFrame(tick)
    }
    setProgress(0)
    frame = requestAnimationFrame(tick)
    return () => cancelAnimationFrame(frame)
  }, [items, duration])

  const interpolated = useMemo(() => {
    const nextItems = items.map((item, index) => {
      const key = entityKey(item, `entity-${index}`)
      const prev = previous.current[key]
      if (!prev || typeof prev.lat !== 'number' || typeof prev.lng !== 'number' || typeof item.lat !== 'number' || typeof item.lng !== 'number') {
        return item
      }
      return {
        ...item,
        lat: prev.lat + ((item.lat as number) - prev.lat) * progress,
        lng: prev.lng + ((item.lng as number) - prev.lng) * progress,
      }
    })
    previous.current = Object.fromEntries(items.map((item, index) => [entityKey(item, `entity-${index}`), item]))
    return nextItems
  }, [items, progress])

  return interpolated
}

