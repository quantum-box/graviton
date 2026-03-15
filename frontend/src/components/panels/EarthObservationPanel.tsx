'use client'

import { CompactList, PanelCard } from './common'
import { useTranslation } from 'react-i18next'

export function EarthObservationPanel({ earthquakes, fires, spaceWeather, weather }: { earthquakes: any[]; fires: any[]; spaceWeather: any; weather: any }) {
  const { t } = useTranslation()

  return (
    <div className="space-y-3">
      <PanelCard title={t('panels.earthquakesTitle')} subtitle={t('panels.earthquakesSubtitle')}>
        <CompactList items={earthquakes.map((q) => ({ title: `${q.place}`, meta: `M${q.magnitude ?? q.mag}` }))} />
      </PanelCard>
      <PanelCard title={t('panels.firesTitle')} subtitle={t('panels.firesSubtitle')}>
        <CompactList items={fires.map((fire) => ({ title: `${fire.acq_date || ''} ${fire.acq_time || ''}`.trim() || t('panels.hotspot'), meta: `FRP ${fire.frp} | ${fire.confidence}` }))} />
      </PanelCard>
      <PanelCard title={t('panels.spaceWeatherTitle')} subtitle={t('panels.spaceWeatherSubtitle')}>
        <div className="text-sm">Kp {spaceWeather?.kp_index ?? '-'} | {spaceWeather?.status || spaceWeather?.kp_text || t('panels.unknown')}</div>
      </PanelCard>
      <PanelCard title={t('panels.weatherTitle')} subtitle={t('panels.weatherSubtitle')}>
        <div className="text-sm text-slate-400">{weather?.radar_tile_path ? t('panels.weatherAvailable') : t('panels.weatherMissing')}</div>
      </PanelCard>
    </div>
  )
}
