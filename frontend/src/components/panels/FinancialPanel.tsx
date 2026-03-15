'use client'

import { CompactList, PanelCard } from './common'
import { useTranslation } from 'react-i18next'

export function FinancialPanel({ stocks, oil, celebrity }: { stocks: any[]; oil: any[]; celebrity: { trackedFlights: any[]; yachts: any[] } }) {
  const { t } = useTranslation()

  return (
    <div className="space-y-3">
      <PanelCard title={t('panels.defenseStocks')} subtitle={t('panels.defenseStocksSubtitle')}>
        <CompactList items={stocks.map((stock) => ({ title: `${stock.symbol} ${stock.price}`, meta: `${stock.name} | ${stock.change_pct}%` }))} />
      </PanelCard>
      <PanelCard title={t('panels.oilTitle')} subtitle={t('panels.oilSubtitle')}>
        <CompactList items={oil.map((item) => ({ title: `${item.name} ${item.price}`, meta: `${item.change_pct}%` }))} />
      </PanelCard>
      <PanelCard title={t('panels.celebrityTitle')} subtitle={t('panels.celebritySubtitle')}>
        <div className="text-sm text-slate-400">{t('panels.celebrityStats', { aircraft: celebrity.trackedFlights.length, yachts: celebrity.yachts.length })}</div>
      </PanelCard>
    </div>
  )
}
