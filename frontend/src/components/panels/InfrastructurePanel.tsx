'use client'

import { CompactList, PanelCard } from './common'

export function InfrastructurePanel({ outages, kiwisdr, datacenters, cctv }: { outages: any[]; kiwisdr: any[]; datacenters: any[]; cctv: any[] }) {
  return (
    <div className="space-y-3">
      <PanelCard title="Internet Outages" subtitle="IODA">
        <CompactList items={outages.map((item) => ({ title: item.region_name || item.region || 'Outage', meta: `${item.country_name || item.country || ''} | ${item.severity}` }))} />
      </PanelCard>
      <PanelCard title="Data Centers" subtitle="DC map dataset">
        <CompactList items={datacenters.map((item) => ({ title: item.name, meta: `${item.company || ''} | ${item.city || ''}` }))} />
      </PanelCard>
      <PanelCard title="CCTV" subtitle="TfL / Singapore">
        <CompactList items={cctv.map((item) => ({ title: item.source, meta: item.direction || item.id, href: item.media_url }))} />
      </PanelCard>
      <PanelCard title="KiwiSDR" subtitle="Public receivers">
        <CompactList items={kiwisdr.map((item) => ({ title: item.name, meta: `${item.users ?? 0}/${item.users_max ?? 0} users`, href: item.url }))} />
      </PanelCard>
    </div>
  )
}
