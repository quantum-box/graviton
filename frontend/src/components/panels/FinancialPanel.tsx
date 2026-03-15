'use client'

import { CompactList, PanelCard } from './common'

export function FinancialPanel({ stocks, oil, celebrity }: { stocks: any[]; oil: any[]; celebrity: { trackedFlights: any[]; yachts: any[] } }) {
  return (
    <div className="space-y-3">
      <PanelCard title="Defense Stocks" subtitle="Yahoo Finance">
        <CompactList items={stocks.map((stock) => ({ title: `${stock.symbol} ${stock.price}`, meta: `${stock.name} | ${stock.change_pct}%` }))} />
      </PanelCard>
      <PanelCard title="Oil" subtitle="WTI / Brent">
        <CompactList items={oil.map((item) => ({ title: `${item.name} ${item.price}`, meta: `${item.change_pct}%` }))} />
      </PanelCard>
      <PanelCard title="Celebrity Tracking" subtitle="PlaneAlert / YachtAlert">
        <div className="text-sm text-slate-400">Tracked aircraft {celebrity.trackedFlights.length} | tracked yachts {celebrity.yachts.length}</div>
      </PanelCard>
    </div>
  )
}
