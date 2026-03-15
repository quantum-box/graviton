'use client'

import { useEffect, useState } from 'react'
import { PanelCard } from './common'

export function GeocodingPanel({ focusLocation, entity }: { focusLocation: { lat: number; lng: number; label?: string } | null; entity: any }) {
  const [reverse, setReverse] = useState<any>(null)
  const target = focusLocation || (entity?.lat && entity?.lng ? { lat: entity.lat, lng: entity.lng, label: entity.name || entity.title || entity.callsign } : null)

  useEffect(() => {
    let mounted = true
    async function run() {
      if (!target) return
      const resp = await fetch(`/api/geocode/reverse?lat=${target.lat}&lng=${target.lng}`)
      const data = await resp.json()
      if (mounted) setReverse(data)
    }
    run()
    return () => {
      mounted = false
    }
  }, [target?.lat, target?.lng])

  return (
    <PanelCard title="Geocoding" subtitle="Nominatim reverse geocode">
      {target ? (
        <>
          <div className="rounded-lg bg-black/20 p-2">
            <div className="font-semibold">{target.label || 'Focused location'}</div>
            <div className="text-slate-400">{target.lat.toFixed(4)}, {target.lng.toFixed(4)}</div>
          </div>
          <div className="text-sm text-slate-300">{reverse?.display_name || 'Resolving location...'}</div>
        </>
      ) : (
        <div className="text-sm text-slate-500">Use the geocode search in the toolbar or select a map entity.</div>
      )}
    </PanelCard>
  )
}
