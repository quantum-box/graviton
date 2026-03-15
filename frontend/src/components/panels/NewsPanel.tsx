'use client'

import { CompactList, PanelCard } from './common'
import { useTranslation } from 'react-i18next'

export function NewsPanel({ news, gdelt }: { news: any[]; gdelt: any[] }) {
  const { t } = useTranslation()

  return (
    <div className="space-y-3">
      <PanelCard title={t('panels.newsTitle')} subtitle={t('panels.newsSubtitle')}>
        <CompactList items={news.map((item) => ({ title: item.title, meta: `${item.source} | ${t('panels.risk', { count: item.risk_score })}`, href: item.url }))} />
      </PanelCard>
      <PanelCard title={t('panels.gdeltTitle')} subtitle={t('panels.gdeltSubtitle')}>
        <CompactList items={gdelt.map((item) => ({ title: item.title || item.description || t('panels.incident'), meta: t('panels.mentions', { count: item.num_mentions ?? 0 }), href: item.source_url }))} />
      </PanelCard>
    </div>
  )
}
