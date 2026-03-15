'use client'

import { CompactList, PanelCard } from './common'

export function NewsPanel({ news, gdelt }: { news: any[]; gdelt: any[] }) {
  return (
    <div className="space-y-3">
      <PanelCard title="News" subtitle="RSS / clustering / risk">
        <CompactList items={news.map((item) => ({ title: item.title, meta: `${item.source} | risk ${item.risk_score}`, href: item.url }))} />
      </PanelCard>
      <PanelCard title="GDELT" subtitle="Global incident heat">
        <CompactList items={gdelt.map((item) => ({ title: item.title || item.description || 'Incident', meta: `mentions ${item.num_mentions ?? 0}`, href: item.source_url }))} />
      </PanelCard>
    </div>
  )
}
