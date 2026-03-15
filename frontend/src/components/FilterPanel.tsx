'use client'

import { SlidersHorizontal } from 'lucide-react'
import { useMemo, useState } from 'react'
import { AdvancedFilterModal } from '@/components/AdvancedFilterModal'
import type { DashboardData } from '@/types/dashboard'

export interface FilterState {
  militaryTypes: string[]
  trackedCategories: string[]
  yachtCategories: string[]
}

export function FilterPanel({
  data,
  filters,
  onChange,
}: {
  data: DashboardData
  filters: FilterState
  onChange: (filters: FilterState) => void
}) {
  const [modal, setModal] = useState<null | keyof FilterState>(null)

  const options = useMemo(
    () => ({
      militaryTypes: Array.from(
        new Set((data.fastData?.military_flights ?? []).map((item: any) => item.military_type).filter(Boolean)),
      ).sort(),
      trackedCategories: Array.from(
        new Set((data.fastData?.tracked_flights ?? []).map((item: any) => item.alert_category).filter(Boolean)),
      ).sort(),
      yachtCategories: Array.from(
        new Set((data.fastData?.ships ?? []).map((item: any) => item.yacht_category).filter(Boolean)),
      ).sort(),
    }),
    [data],
  )

  const config = {
    militaryTypes: 'Military Types',
    trackedCategories: 'PlaneAlert Categories',
    yachtCategories: 'YachtAlert Categories',
  }

  return (
    <>
      <div className="rounded-2xl border border-white/10 bg-black/45 p-3 backdrop-blur-md">
        <div className="mb-3 flex items-center gap-2 text-[10px] uppercase tracking-[0.22em] text-cyan-300">
          <SlidersHorizontal size={14} />
          Filters
        </div>
        <div className="space-y-2">
          {(Object.keys(config) as Array<keyof FilterState>).map((key) => (
            <button
              key={key}
              onClick={() => setModal(key)}
              className="flex w-full items-center justify-between rounded-2xl border border-white/10 bg-white/5 px-3 py-2 text-left text-sm text-slate-200 hover:bg-white/8"
            >
              <span>{config[key]}</span>
              <span className="text-xs text-slate-500">{filters[key].length || 'All'}</span>
            </button>
          ))}
        </div>
      </div>
      {(Object.keys(config) as Array<keyof FilterState>).map((key) => (
        <AdvancedFilterModal
          key={key}
          open={modal === key}
          title={config[key]}
          options={options[key]}
          selected={filters[key]}
          onClose={() => setModal(null)}
          onChange={(next) => onChange({ ...filters, [key]: next })}
        />
      ))}
    </>
  )
}

