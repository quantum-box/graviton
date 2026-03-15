'use client'

import { CompactList, PanelCard } from './common'
import { getFlightOperator } from '@/utils/airlineCodes'
import { useTranslation } from 'react-i18next'

export function FlightPanel({ entity, tracked }: { entity: any; tracked: any[] }) {
  const operator = getFlightOperator(entity) || entity?.registration || entity?.model
  const { t } = useTranslation()

  return (
    <PanelCard title={t('panels.flightsTitle')} subtitle={t('panels.flightsSubtitle')}>
      {entity?.sourceType?.includes('flight') || entity?.sourceType === 'uavs' ? (
        <div className="rounded-lg bg-black/20 p-2 text-sm">
          <div className="font-semibold">{entity.callsign || entity.icao24 || t('panels.selectedAircraft')}</div>
          <div className="text-slate-400">{operator}</div>
        </div>
      ) : (
        <div className="text-sm text-slate-500">{t('panels.selectAircraft')}</div>
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
