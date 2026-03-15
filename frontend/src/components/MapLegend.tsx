'use client'

import type { LayerVisibility } from '@/app/page'
import { AircraftIcon } from '@/components/map/icons/AircraftIcons'
import { SatelliteIcon } from '@/components/map/icons/SatelliteIcons'
import { useTranslation } from 'react-i18next'

export function MapLegend({ layers }: { layers: LayerVisibility }) {
  const { t } = useTranslation()
  const active = Object.entries(layers).filter(([, enabled]) => enabled)
  return (
    <div className="rounded-2xl border border-white/10 bg-black/45 p-3 backdrop-blur-md">
      <div className="mb-2 text-[10px] uppercase tracking-[0.22em] text-cyan-300">{t('legend.title')}</div>
      <div className="mb-2 flex items-center gap-3 text-slate-300">
        <div className="flex items-center gap-1 text-xs"><AircraftIcon /> {t('legend.aircraft')}</div>
        <div className="flex items-center gap-1 text-xs"><SatelliteIcon /> {t('legend.satellite')}</div>
      </div>
      <div className="flex max-w-[16rem] flex-wrap gap-1.5">
        {active.map(([key]) => (
          <span key={key} className="rounded-full border border-white/10 bg-white/5 px-2 py-1 text-[11px] text-slate-300">
            {t(`layers.${key as keyof LayerVisibility}`)}
          </span>
        ))}
      </div>
    </div>
  )
}
