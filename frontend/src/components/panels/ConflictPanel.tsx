'use client'

import { CompactList, PanelCard } from './common'

export function ConflictPanel({ liveuamap, frontlines, gdelt }: { liveuamap: any[]; frontlines: any; gdelt: any[] }) {
  return (
    <div className="space-y-3">
      <PanelCard title="LiveUAmap" subtitle="Conflict / frontline incidents">
        <CompactList items={liveuamap.map((item) => ({ title: item.title || 'Conflict event', meta: item.region, href: item.link }))} />
      </PanelCard>
      <PanelCard title="Frontlines" subtitle="DeepState mirror">
        <div className="text-sm text-slate-400">Features: {frontlines?.features?.length ?? 0}</div>
      </PanelCard>
      <PanelCard title="Related GDELT" subtitle="Correlated conflict media">
        <CompactList items={gdelt.map((item) => ({ title: item.title || item.description || 'Incident', meta: item.date }))} />
      </PanelCard>
    </div>
  )
}
