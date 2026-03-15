'use client'

import { CompactList, PanelCard } from './common'
import { useTranslation } from 'react-i18next'

export function ShipPanel({ entity, ships }: { entity: any; ships: any[] }) {
  const yachts = ships.filter((ship) => ship.yacht_alert)
  const { t } = useTranslation()
  return (
    <PanelCard title={t('panels.shipsTitle')} subtitle={t('panels.shipsSubtitle')}>
      {entity?.sourceType === 'ships' ? (
        <div className="rounded-lg bg-black/20 p-2">
          <div className="font-semibold">{entity.name || entity.mmsi}</div>
          <div className="text-slate-400">{entity.type || entity.country}</div>
        </div>
      ) : (
        <div className="text-sm text-slate-500">{t('panels.shipsEmpty')}</div>
      )}
      <CompactList items={yachts.map((ship) => ({ title: ship.yacht_name || ship.name, meta: ship.yacht_owner || ship.country, href: ship.yacht_link }))} />
    </PanelCard>
  )
}
