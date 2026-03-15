'use client'

import { CompactList, PanelCard } from './common'
import { useTranslation } from 'react-i18next'

export function ConflictPanel({ liveuamap, frontlines, gdelt }: { liveuamap: any[]; frontlines: any; gdelt: any[] }) {
  const { t } = useTranslation()

  return (
    <div className="space-y-3">
      <PanelCard title={t('panels.liveuaTitle')} subtitle={t('panels.liveuaSubtitle')}>
        <CompactList items={liveuamap.map((item) => ({ title: item.title || t('panels.conflictEvent'), meta: item.region, href: item.link }))} />
      </PanelCard>
      <PanelCard title={t('panels.frontlinesTitle')} subtitle={t('panels.frontlinesSubtitle')}>
        <div className="text-sm text-slate-400">{t('panels.features', { count: frontlines?.features?.length ?? 0 })}</div>
      </PanelCard>
      <PanelCard title={t('panels.relatedGdelt')} subtitle={t('panels.relatedGdeltSubtitle')}>
        <CompactList items={gdelt.map((item) => ({ title: item.title || item.description || t('panels.incident'), meta: item.date }))} />
      </PanelCard>
    </div>
  )
}
