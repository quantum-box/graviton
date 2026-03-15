'use client'

import { CompactList, PanelCard } from './common'
import { useTranslation } from 'react-i18next'

export function SatellitePanel({ entity, satellites }: { entity: any; satellites: any[] }) {
  const { t } = useTranslation()

  return (
    <PanelCard title={t('panels.satellitesTitle')} subtitle={t('panels.satellitesSubtitle')}>
      {entity?.sourceType === 'satellites' ? (
        <div className="rounded-lg bg-black/20 p-2">
          <div className="font-semibold">{entity.name}</div>
          <div className="text-slate-400">{entity.mission} | {entity.country}</div>
        </div>
      ) : (
        <div className="text-sm text-slate-500">{t('panels.satellitesEmpty')}</div>
      )}
      <CompactList items={satellites.map((sat) => ({ title: sat.name, meta: `${sat.mission} | ${sat.country}` }))} />
    </PanelCard>
  )
}
