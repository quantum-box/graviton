'use client'

import { createContext, useContext } from 'react'
import type { BaseEntity, DashboardData } from '@/types/dashboard'

interface DashboardDataContextValue extends DashboardData {
  selectedEntity: BaseEntity | null
  setSelectedEntity: (entity: BaseEntity | null) => void
}

const DashboardDataContext = createContext<DashboardDataContextValue | null>(null)

export function DashboardDataProvider({
  children,
  fastData,
  slowData,
  selectedEntity,
  setSelectedEntity,
}: DashboardDataContextValue & { children: React.ReactNode }) {
  return (
    <DashboardDataContext.Provider value={{ fastData, slowData, selectedEntity, setSelectedEntity }}>
      {children}
    </DashboardDataContext.Provider>
  )
}

export function useDashboardData() {
  const context = useContext(DashboardDataContext)
  if (!context) throw new Error('useDashboardData must be used within DashboardDataProvider')
  return context
}

