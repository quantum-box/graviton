'use client'

import { CompactList, PanelCard } from './common'
import { getFlightOperator } from '@/utils/airlineCodes'

export function FlightPanel({ entity, tracked }: { entity: any; tracked: any[] }) {
  const operator = getFlightOperator(entity) || entity?.registration || entity?.model

  return (
    <PanelCard title="Flights" subtitle="ADS-B / OpenSky / military / GPS jamming">
      {entity?.sourceType?.includes('flight') || entity?.sourceType === 'uavs' ? (
        <div className="rounded-lg bg-black/20 p-2 text-sm">
          <div className="font-semibold">{entity.callsign || entity.icao24 || 'Selected aircraft'}</div>
          <div className="text-slate-400">{operator}</div>
        </div>
      ) : (
        <div className="text-sm text-slate-500">Select an aircraft on the map to inspect its metadata.</div>
      )}
      <CompactList
        items={tracked.map((flight: any) => ({
          title: flight.tracked_name || flight.alert_operator || flight.callsign || flight.icao24,
          meta: [getFlightOperator(flight), flight.alert_category, flight.registration].filter(Boolean).join(' | '),
        }))}
      />
    </PanelCard>
  )
}
