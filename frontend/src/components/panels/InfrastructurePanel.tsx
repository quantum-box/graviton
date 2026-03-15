'use client'

import { CompactList, PanelCard } from './common'
import { useTranslation } from 'react-i18next'

export function InfrastructurePanel({ outages, kiwisdr, datacenters, cctv }: { outages: any[]; kiwisdr: any[]; datacenters: any[]; cctv: any[] }) {
  const { t } = useTranslation()

  return (
    <div className="space-y-3">
      <PanelCard title={t('panels.internetOutages')} subtitle={t('panels.internetOutagesSubtitle')}>
        <CompactList items={outages.map((item) => ({ title: item.region_name || item.region || t('panels.outage'), meta: `${item.country_name || item.country || ''} | ${item.severity}` }))} />
      </PanelCard>
      <PanelCard title={t('panels.dataCenters')} subtitle={t('panels.dataCentersSubtitle')}>
        <CompactList items={datacenters.map((item) => ({ title: item.name, meta: `${item.company || ''} | ${item.city || ''}` }))} />
      </PanelCard>
      <PanelCard title={t('panels.cctvTitle')} subtitle={t('panels.cctvSubtitle')}>
        <CompactList items={cctv.map((item) => ({ title: item.source, meta: item.direction || item.id, href: item.media_url }))} />
      </PanelCard>
      <PanelCard title={t('panels.kiwisdrTitle')} subtitle={t('panels.kiwisdrSubtitle')}>
        <CompactList items={kiwisdr.map((item) => ({ title: item.name, meta: t('panels.users', { current: item.users ?? 0, max: item.users_max ?? 0 }), href: item.url }))} />
      </PanelCard>
    </div>
  )
}
