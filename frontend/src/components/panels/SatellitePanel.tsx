'use client'

import { CompactList, PanelCard } from './common'

export function SatellitePanel({ entity, satellites }: { entity: any; satellites: any[] }) {
  return (
    <PanelCard title="Satellites" subtitle="SGP4 / Sentinel catalog">
      {entity?.sourceType === 'satellites' ? (
        <div className="rounded-lg bg-black/20 p-2">
          <div className="font-semibold">{entity.name}</div>
          <div className="text-slate-400">{entity.mission} | {entity.country}</div>
        </div>
      ) : (
        <div className="text-sm text-slate-500">Tracked satellites are pseudo-projected from cached orbital elements for continuous situational awareness.</div>
      )}
      <CompactList items={satellites.map((sat) => ({ title: sat.name, meta: `${sat.mission} | ${sat.country}` }))} />
    </PanelCard>
  )
}
