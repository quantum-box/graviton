'use client'

import { startTransition, useDeferredValue, useEffect, useMemo, useRef, useState } from 'react'
import dynamic from 'next/dynamic'
import { AnimatePresence, motion } from 'framer-motion'
import type { LucideIcon } from 'lucide-react'
import {
  BookOpen,
  CircleHelp,
  Filter,
  GripVertical,
  Info,
  Layers3,
  Map,
  Menu,
  MousePointer2,
  PanelLeftClose,
  PanelLeftOpen,
  PanelRightClose,
  PanelRightOpen,
  Ruler,
  Search,
  Settings2,
  SquareMousePointer,
  Workflow,
  X,
} from 'lucide-react'
import { Group, Panel, Separator, type PanelImperativeHandle } from 'react-resizable-panels'
import { useTranslation } from 'react-i18next'
import { LeftPanel } from '@/components/LeftPanel'
import { RightPanel } from '@/components/RightPanel'
import { StatusBar } from '@/components/StatusBar'
import { FilterPanel, type FilterState } from '@/components/FilterPanel'
import { FindLocateBar } from '@/components/FindLocateBar'
import { MapLegend } from '@/components/MapLegend'
import { SettingsPanel } from '@/components/SettingsPanel'
import { RadioInterceptPanel } from '@/components/RadioInterceptPanel'
import { useDataPolling } from '@/hooks/useDataPolling'
import { DashboardDataProvider } from '@/lib/DashboardDataContext'
import { OnboardingModal } from '@/components/OnboardingModal'
import { ChangelogModal } from '@/components/ChangelogModal'
import { ErrorBoundary } from '@/components/ErrorBoundary'
import i18n from '@/lib/i18n'

const MapView = dynamic(() => import('@/components/MapView'), { ssr: false })
const NavigatorMiniMap = dynamic(() => import('@/components/NavigatorMiniMap'), { ssr: false })

export interface LayerVisibility {
  commercial_flights: boolean
  private_flights: boolean
  private_jets: boolean
  military_flights: boolean
  tracked_flights: boolean
  uavs: boolean
  gps_jamming: boolean
  ships: boolean
  satellites: boolean
  earthquakes: boolean
  news: boolean
  firms_fires: boolean
  gdelt: boolean
  liveuamap: boolean
  frontlines: boolean
  internet_outages: boolean
  kiwisdr: boolean
  datacenters: boolean
  cctv: boolean
  weather_radar: boolean
  day_night: boolean
}

const DEFAULT_LAYERS: LayerVisibility = {
  commercial_flights: true,
  private_flights: true,
  private_jets: true,
  military_flights: true,
  tracked_flights: true,
  uavs: true,
  gps_jamming: true,
  ships: true,
  satellites: true,
  earthquakes: true,
  news: true,
  firms_fires: false,
  gdelt: true,
  liveuamap: true,
  frontlines: true,
  internet_outages: false,
  kiwisdr: false,
  datacenters: false,
  cctv: false,
  weather_radar: false,
  day_night: true,
}

const DEFAULT_FILTERS: FilterState = {
  militaryTypes: [],
  trackedCategories: [],
  yachtCategories: [],
}

const MENU_ITEMS = ['file', 'edit', 'view', 'dataSources', 'tools', 'window', 'help'] as const
const INSPECTOR_TABS = ['layers', 'properties', 'info'] as const
const LANGUAGE_OPTIONS = [
  { code: 'ja', label: '日本語' },
  { code: 'en', label: 'English' },
  { code: 'zh', label: '中文' },
  { code: 'ko', label: '한국어' },
] as const

type ToolId = 'select' | 'measure' | 'find' | 'marker' | 'filter'
type FloatingPanelId = 'legend' | 'navigator' | 'filters' | 'radio'
type FloatingPanelState = { x: number; y: number; docked: boolean; collapsed: boolean }

const TOOL_ITEMS: Array<{ id: ToolId; shortcut: string; icon: LucideIcon }> = [
  { id: 'select', shortcut: 'V', icon: MousePointer2 },
  { id: 'measure', shortcut: 'M', icon: Ruler },
  { id: 'find', shortcut: 'F', icon: Search },
  { id: 'marker', shortcut: 'K', icon: SquareMousePointer },
  { id: 'filter', shortcut: 'Shift+F', icon: Filter },
]

const FLOATING_PANEL_DEFAULTS: Record<FloatingPanelId, FloatingPanelState> = {
  legend: { x: 24, y: 24, docked: false, collapsed: false },
  navigator: { x: 24, y: 286, docked: false, collapsed: false },
  filters: { x: 320, y: 24, docked: false, collapsed: false },
  radio: { x: 320, y: 318, docked: true, collapsed: false },
}

function formatEntityValue(value: unknown) {
  if (value === null || value === undefined || value === '') return i18n.t('common.na')
  if (typeof value === 'number') return Number.isInteger(value) ? value.toString() : value.toFixed(3)
  if (typeof value === 'boolean') return value ? i18n.t('common.yes') : i18n.t('common.no')
  return String(value)
}

function SplashScreen() {
  const { t } = useTranslation()

  return (
    <motion.div
      className="absolute inset-0 z-[120] flex items-center justify-center bg-[#141414]"
      initial={{ opacity: 1 }}
      exit={{ opacity: 0, transition: { duration: 0.45, ease: 'easeInOut' } }}
    >
      <motion.div
        className="flex flex-col items-center gap-4"
        initial={{ opacity: 0, scale: 0.92, y: 18 }}
        animate={{ opacity: 1, scale: 1, y: 0 }}
        exit={{ opacity: 0, scale: 1.04 }}
        transition={{ duration: 0.55, ease: [0.19, 1, 0.22, 1] }}
      >
        <div className="flex h-20 w-20 items-center justify-center rounded-[22px] border border-white/10 bg-[linear-gradient(180deg,#363636,#1b1b1b)] shadow-[0_28px_55px_rgba(0,0,0,0.55)]">
          <Map size={34} className="text-[var(--accent)]" />
        </div>
        <div className="text-center">
          <div className="font-[family-name:IBM_Plex_Sans] text-[11px] uppercase tracking-[0.48em] text-[var(--text-muted)]">{t('app.subtitle')}</div>
          <div className="mt-2 text-3xl font-semibold tracking-[0.24em] text-white">{t('app.title').toUpperCase()}</div>
        </div>
      </motion.div>
    </motion.div>
  )
}

function FloatingWorkspacePanel({
  title,
  badge,
  state,
  onUpdate,
  children,
  width = 272,
}: {
  title: string
  badge?: string
  state: FloatingPanelState
  onUpdate: (next: FloatingPanelState) => void
  children: React.ReactNode
  width?: number
}) {
  const dragRef = useRef<{ startX: number; startY: number; originX: number; originY: number } | null>(null)
  const { t } = useTranslation()

  useEffect(() => {
    return () => {
      dragRef.current = null
    }
  }, [])

  if (state.docked) return null

  return (
    <motion.section
      initial={{ opacity: 0, scale: 0.98, y: 8 }}
      animate={{ opacity: 1, scale: 1, y: 0 }}
      transition={{ duration: 0.18, ease: 'easeOut' }}
      className="absolute z-20 overflow-hidden rounded-[10px] border border-[var(--panel-border)] bg-[var(--panel-bg)] shadow-[0_24px_60px_rgba(0,0,0,0.45)]"
      style={{ left: state.x, top: state.y, width }}
    >
      <div
        className="flex cursor-grab items-center justify-between border-b border-[var(--panel-border)] bg-[var(--panel-header)] px-3 py-2 active:cursor-grabbing"
        onPointerDown={(event) => {
          const target = event.target as HTMLElement
          if (target.closest('[data-panel-action="true"]')) return
          dragRef.current = {
            startX: event.clientX,
            startY: event.clientY,
            originX: state.x,
            originY: state.y,
          }

          const handleMove = (moveEvent: PointerEvent) => {
            if (!dragRef.current) return
            onUpdate({
              ...state,
              x: Math.max(8, dragRef.current.originX + moveEvent.clientX - dragRef.current.startX),
              y: Math.max(8, dragRef.current.originY + moveEvent.clientY - dragRef.current.startY),
            })
          }

          const handleUp = () => {
            dragRef.current = null
            window.removeEventListener('pointermove', handleMove)
            window.removeEventListener('pointerup', handleUp)
          }

          window.addEventListener('pointermove', handleMove)
          window.addEventListener('pointerup', handleUp)
        }}
      >
        <div className="flex items-center gap-2">
          <GripVertical size={14} className="text-[var(--text-muted)]" />
          <div className="text-[11px] uppercase tracking-[0.24em] text-[var(--text-muted)]">{title}</div>
          {badge && <span className="rounded-sm bg-black/30 px-1.5 py-0.5 text-[10px] font-mono text-[var(--accent)]">{badge}</span>}
        </div>
        <div className="flex items-center gap-1">
          <button
            data-panel-action="true"
            onClick={() => onUpdate({ ...state, collapsed: !state.collapsed })}
            className="rounded-sm px-1.5 py-0.5 text-[11px] text-[var(--text-secondary)] hover:bg-white/6"
          >
            {state.collapsed ? t('shell.open') : t('shell.fold')}
          </button>
          <button
            data-panel-action="true"
            onClick={() => onUpdate({ ...state, docked: true, collapsed: false })}
            className="rounded-sm px-1.5 py-0.5 text-[11px] text-[var(--text-secondary)] hover:bg-white/6"
          >
            {t('shell.dock')}
          </button>
        </div>
      </div>
      <AnimatePresence initial={false}>
        {!state.collapsed && (
          <motion.div
            initial={{ height: 0, opacity: 0 }}
            animate={{ height: 'auto', opacity: 1 }}
            exit={{ height: 0, opacity: 0 }}
            transition={{ duration: 0.18, ease: 'easeOut' }}
            className="overflow-hidden"
          >
            <div className="max-h-[320px] overflow-auto p-3">{children}</div>
          </motion.div>
        )}
      </AnimatePresence>
    </motion.section>
  )
}

function PropertiesSidebar({
  tab,
  onTabChange,
  layers,
  counts,
  onToggleLayer,
  selectedEntity,
  fastData,
  slowData,
  focusLocation,
}: {
  tab: (typeof INSPECTOR_TABS)[number]
  onTabChange: (tab: (typeof INSPECTOR_TABS)[number]) => void
  layers: LayerVisibility
  counts: Record<string, number>
  onToggleLayer: (key: keyof LayerVisibility) => void
  selectedEntity: any
  fastData: any
  slowData: any
  focusLocation: { lat: number; lng: number; label?: string } | null
}) {
  const { t } = useTranslation()
  const propertyPairs = useMemo(
    () =>
      Object.entries(selectedEntity ?? {})
        .filter(([key]) => !['description', 'sourceType'].includes(key))
        .slice(0, 14),
    [selectedEntity],
  )

  return (
    <div className="flex h-full flex-col bg-[var(--panel-bg)]">
      <div className="flex border-b border-[var(--panel-border)] bg-[var(--panel-header)]">
        {INSPECTOR_TABS.map((item) => (
          <button
            key={item}
            onClick={() => onTabChange(item)}
            className={`border-r border-[var(--panel-border)] px-4 py-3 text-[11px] uppercase tracking-[0.22em] ${
              tab === item ? 'bg-[#242424] text-[var(--accent)]' : 'text-[var(--text-muted)]'
            }`}
          >
            {t(`inspector.${item}`)}
          </button>
        ))}
      </div>

      <div className="flex-1 overflow-hidden">
        {tab === 'layers' && (
          <div className="flex h-full flex-col">
            <div className="grid grid-cols-3 gap-px border-b border-[var(--panel-border)] bg-[var(--panel-border)]">
              <div className="bg-[#202020] px-3 py-2 text-[10px] uppercase tracking-[0.2em] text-[var(--text-muted)]">{t('inspector.visible')}</div>
              <div className="bg-[#202020] px-3 py-2 text-[10px] uppercase tracking-[0.2em] text-[var(--text-muted)]">{t('inspector.category')}</div>
              <div className="bg-[#202020] px-3 py-2 text-right text-[10px] uppercase tracking-[0.2em] text-[var(--text-muted)]">{t('inspector.count')}</div>
            </div>
            <LeftPanel layers={layers} counts={counts} onToggle={onToggleLayer} className="w-full flex-1 border-r-0" />
          </div>
        )}

        {tab === 'properties' && (
          <div className="h-full overflow-auto p-4">
            <div className="rounded-[10px] border border-[var(--panel-border)] bg-[#1c1c1c]">
              <div className="border-b border-[var(--panel-border)] px-4 py-3">
                <div className="text-[10px] uppercase tracking-[0.24em] text-[var(--text-muted)]">{t('inspector.selectedObject')}</div>
                <div className="mt-2 text-base font-medium text-[var(--text-primary)]">
                  {selectedEntity?.title || selectedEntity?.name || selectedEntity?.callsign || selectedEntity?.icao24 || t('inspector.nothingSelected')}
                </div>
                <div className="mt-1 text-xs text-[var(--text-secondary)]">{selectedEntity?.sourceType || t('inspector.useMapToInspect')}</div>
              </div>
              <div className="grid grid-cols-2 gap-px bg-[var(--panel-border)]">
                {propertyPairs.length > 0 ? (
                  propertyPairs.map(([key, value]) => (
                    <div key={key} className="bg-[#1c1c1c] px-4 py-2">
                      <div className="text-[10px] uppercase tracking-[0.16em] text-[var(--text-muted)]">{key.replaceAll('_', ' ')}</div>
                      <div className="mt-1 text-sm text-[var(--text-primary)]">{formatEntityValue(value)}</div>
                    </div>
                  ))
                ) : (
                  <div className="col-span-2 px-4 py-6 text-sm text-[var(--text-secondary)]">{t('inspector.noMetadata')}</div>
                )}
              </div>
            </div>
          </div>
        )}

        {tab === 'info' && <RightPanel selectedEntity={selectedEntity} fastData={fastData} slowData={slowData} focusLocation={focusLocation} className="h-full w-full" />}
      </div>
    </div>
  )
}

function MobileDashboard({
  layers,
  counts,
  onToggleLayer,
  data,
  filters,
  onFiltersChange,
  focusLocation,
  onSearchSelect,
  onSelectEntity,
  onMouseMove,
  zoom,
  onZoomChange,
  activeTool,
  selectedEntity,
}: {
  layers: LayerVisibility
  counts: Record<string, number>
  onToggleLayer: (key: keyof LayerVisibility) => void
  data: { fastData: any; slowData: any }
  filters: FilterState
  onFiltersChange: (filters: FilterState) => void
  focusLocation: { lat: number; lng: number; label?: string } | null
  onSearchSelect: (result: { lat: number; lng: number; label?: string } | null) => void
  onSelectEntity: (entity: any) => void
  onMouseMove: (coords: { lat: number; lng: number } | null) => void
  zoom: number
  onZoomChange: (zoom: number) => void
  activeTool: ToolId
  selectedEntity: any
}) {
  const [section, setSection] = useState<'layers' | 'filters' | 'intel'>('layers')
  const { t } = useTranslation()

  return (
    <div className="flex h-screen flex-col bg-[var(--bg-primary)] md:hidden">
      <div className="border-b border-[var(--panel-border)] bg-[var(--chrome-bg)] px-4 py-3">
        <div className="flex items-center justify-between">
          <div>
            <div className="text-[10px] uppercase tracking-[0.28em] text-[var(--text-muted)]">{t('app.title')}</div>
            <div className="text-lg font-semibold text-[var(--text-primary)]">{t('app.commandWorkspace')}</div>
          </div>
          <div className="rounded-md border border-[var(--panel-border)] bg-[#1b1b1b] px-2 py-1 text-[11px] uppercase tracking-[0.2em] text-[var(--accent)]">{t(`tools.${activeTool}`)}</div>
        </div>
        <div className="mt-3">
          <FindLocateBar onSelect={onSearchSelect} />
        </div>
      </div>

      <div className="border-b border-[var(--panel-border)] bg-[var(--panel-header)] px-3 py-2">
        <div className="flex gap-2 overflow-auto">
          {TOOL_ITEMS.map(({ id, icon: Icon }) => (
            <button
              key={id}
              onClick={() => setSection(id === 'filter' ? 'filters' : id === 'find' ? 'intel' : 'layers')}
              className={`flex items-center gap-2 rounded-md border px-3 py-2 text-xs ${
                activeTool === id ? 'border-[var(--accent)] bg-[rgba(255,153,0,0.14)] text-[var(--accent)]' : 'border-[var(--panel-border)] bg-[#222] text-[var(--text-secondary)]'
              }`}
            >
              <Icon size={14} />
              {t(`tools.${id}`)}
            </button>
          ))}
        </div>
      </div>

      <div className="min-h-0 flex-1">
        <MapView
          data={data}
          layers={layers}
          zoom={zoom}
          focusLocation={focusLocation}
          onSelect={onSelectEntity}
          onMouseMove={onMouseMove}
          onZoomChange={onZoomChange}
          activeTool={activeTool}
        />
      </div>

      <div className="grid grid-cols-3 border-t border-[var(--panel-border)] bg-[var(--panel-header)]">
        {(['layers', 'filters', 'intel'] as const).map((item) => (
          <button
            key={item}
            onClick={() => setSection(item)}
            className={`px-3 py-2 text-[11px] uppercase tracking-[0.24em] ${section === item ? 'text-[var(--accent)]' : 'text-[var(--text-muted)]'}`}
          >
            {t(`mobile.${item}`)}
          </button>
        ))}
      </div>

      <motion.div layout className="max-h-[38vh] overflow-auto border-t border-[var(--panel-border)] bg-[var(--panel-bg)] p-3">
        {section === 'layers' && <LeftPanel layers={layers} counts={counts} onToggle={onToggleLayer} className="w-full border-r-0" />}
        {section === 'filters' && (
          <div className="space-y-3">
            <FilterPanel data={data} filters={filters} onChange={onFiltersChange} />
            <MapLegend layers={layers} />
          </div>
        )}
        {section === 'intel' && <RightPanel selectedEntity={selectedEntity} fastData={data.fastData} slowData={data.slowData} focusLocation={focusLocation} className="h-[34vh] w-full" />}
      </motion.div>
    </div>
  )
}

export default function Dashboard() {
  const { t } = useTranslation()
  const [settingsOpen, setSettingsOpen] = useState(false)
  const [onboardingOpen, setOnboardingOpen] = useState(false)
  const [changelogOpen, setChangelogOpen] = useState(false)
  const [splashVisible, setSplashVisible] = useState(true)
  const [menuOpen, setMenuOpen] = useState(false)
  const [leftCollapsed, setLeftCollapsed] = useState(false)
  const [rightCollapsed, setRightCollapsed] = useState(false)
  const [layers, setLayers] = useState<LayerVisibility>(DEFAULT_LAYERS)
  const [selectedEntity, setSelectedEntity] = useState<any>(null)
  const [mouseCoords, setMouseCoords] = useState<{ lat: number; lng: number } | null>(null)
  const [zoom, setZoom] = useState(2.8)
  const [viewState, setViewState] = useState({ lat: 26, lng: 20, zoom: 2.8 })
  const [focusLocation, setFocusLocation] = useState<{ lat: number; lng: number; label?: string } | null>(null)
  const [filters, setFilters] = useState<FilterState>(DEFAULT_FILTERS)
  const [activeTool, setActiveTool] = useState<ToolId>('select')
  const [inspectorTab, setInspectorTab] = useState<(typeof INSPECTOR_TABS)[number]>('properties')
  const [floatingPanels, setFloatingPanels] = useState<Record<FloatingPanelId, FloatingPanelState>>(FLOATING_PANEL_DEFAULTS)
  const leftPanelRef = useRef<PanelImperativeHandle>(null)
  const rightPanelRef = useRef<PanelImperativeHandle>(null)
  const deferredSelectedEntity = useDeferredValue(selectedEntity)
  const { fastData, slowData, isLoading } = useDataPolling()

  const filteredFastData = useMemo(() => {
    if (!fastData) return null
    return {
      ...fastData,
      military_flights:
        filters.militaryTypes.length === 0
          ? fastData.military_flights
          : fastData.military_flights.filter((item: any) => filters.militaryTypes.includes(item.military_type)),
      tracked_flights:
        filters.trackedCategories.length === 0
          ? fastData.tracked_flights
          : fastData.tracked_flights.filter((item: any) => filters.trackedCategories.includes(item.alert_category)),
      ships:
        filters.yachtCategories.length === 0
          ? fastData.ships
          : fastData.ships.filter((item: any) => filters.yachtCategories.includes(item.yacht_category)),
    }
  }, [fastData, filters])

  const counts = useMemo(
    () => ({
      commercial_flights: filteredFastData?.commercial_flights?.length ?? 0,
      private_flights: filteredFastData?.private_flights?.length ?? 0,
      private_jets: filteredFastData?.private_jets?.length ?? 0,
      military_flights: filteredFastData?.military_flights?.length ?? 0,
      tracked_flights: filteredFastData?.tracked_flights?.length ?? 0,
      uavs: filteredFastData?.uavs?.length ?? 0,
      gps_jamming: filteredFastData?.gps_jamming?.length ?? 0,
      ships: filteredFastData?.ships?.length ?? 0,
      satellites: filteredFastData?.satellites?.length ?? 0,
      earthquakes: slowData?.earthquakes?.length ?? 0,
      news: slowData?.news?.length ?? 0,
      firms_fires: slowData?.firms_fires?.length ?? 0,
      gdelt: slowData?.gdelt?.length ?? 0,
      liveuamap: slowData?.liveuamap?.length ?? 0,
      frontlines: slowData?.frontlines?.features?.length ?? 0,
      internet_outages: slowData?.internet_outages?.length ?? 0,
      kiwisdr: slowData?.kiwisdr?.length ?? 0,
      datacenters: slowData?.datacenters?.length ?? 0,
      cctv: slowData?.cctv?.length ?? 0,
    }),
    [filteredFastData, slowData],
  )

  const dashboardData = { fastData: filteredFastData, slowData }

  useEffect(() => {
    const splashTimer = window.setTimeout(() => setSplashVisible(false), 1550)
    const seen = window.localStorage.getItem('graviton-onboarding-seen')
    if (!seen) {
      setOnboardingOpen(true)
      window.localStorage.setItem('graviton-onboarding-seen', 'true')
    }
    return () => window.clearTimeout(splashTimer)
  }, [])

  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      const target = event.target as HTMLElement | null
      const isTyping = target && ['INPUT', 'TEXTAREA'].includes(target.tagName)
      if (isTyping || event.metaKey || event.ctrlKey || event.altKey) return

      if (event.key.toLowerCase() === 'v') setActiveTool('select')
      if (event.key.toLowerCase() === 'm') setActiveTool('measure')
      if (event.key.toLowerCase() === 'f' && !event.shiftKey) setActiveTool('find')
      if (event.key.toLowerCase() === 'f' && event.shiftKey) setActiveTool('filter')
    }

    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [])

  function updateFloatingPanel(id: FloatingPanelId, next: FloatingPanelState) {
    setFloatingPanels((current) => ({ ...current, [id]: next }))
  }

  function toggleLeftPanel() {
    if (leftCollapsed) {
      leftPanelRef.current?.expand()
      setLeftCollapsed(false)
    } else {
      leftPanelRef.current?.collapse()
      setLeftCollapsed(true)
    }
  }

  function toggleRightPanel() {
    if (rightCollapsed) {
      rightPanelRef.current?.expand()
      setRightCollapsed(false)
    } else {
      rightPanelRef.current?.collapse()
      setRightCollapsed(true)
    }
  }

  const statusLabel = isLoading ? t('status.syncing') : focusLocation?.label ? t('status.focus', { label: focusLocation.label.slice(0, 18).toUpperCase() }) : t('status.ready')

  return (
    <DashboardDataProvider fastData={filteredFastData} slowData={slowData} selectedEntity={selectedEntity} setSelectedEntity={setSelectedEntity}>
      <div className="relative h-screen w-screen overflow-hidden bg-[var(--bg-primary)] text-[var(--text-primary)]">
        <AnimatePresence>{splashVisible && <SplashScreen />}</AnimatePresence>

        <div className="hidden h-full flex-col md:flex">
          <div className="desktop-menubar border-b border-[var(--panel-border)]">
            <div className="flex h-8 items-center justify-between px-3">
              <div className="flex items-center gap-3">
                <div className="window-controls">
                  <span className="window-dot close" />
                  <span className="window-dot minimize" />
                  <span className="window-dot maximize" />
                </div>
                <div className="flex items-center gap-2 rounded-md border border-white/6 bg-black/18 px-2 py-1">
                  <Workflow size={14} className="text-[var(--accent)]" />
                  <span className="text-[11px] font-semibold uppercase tracking-[0.34em] text-[var(--text-primary)]">{t('app.title')}</span>
                </div>
                <nav className="flex items-center gap-1">
                  {MENU_ITEMS.map((item) => (
                    <button
                      key={item}
                      onClick={() => {
                        if (item === 'window') {
                          toggleLeftPanel()
                          toggleRightPanel()
                        }
                        if (item === 'help') setOnboardingOpen(true)
                        if (item === 'dataSources') setInspectorTab('layers')
                      }}
                      className="rounded px-2 py-1 text-[12px] text-[var(--text-secondary)] transition hover:bg-white/6 hover:text-[var(--text-primary)]"
                    >
                      {t(`menu.${item}`)}
                    </button>
                  ))}
                </nav>
              </div>
              <div className="flex items-center gap-3 text-[11px] uppercase tracking-[0.22em] text-[var(--text-muted)]">
                <span>{t('app.workspace')}</span>
                <span className="rounded-sm border border-[var(--panel-border)] bg-[#202020] px-2 py-1 text-[var(--accent)]">{t('app.workspaceName')}</span>
                <div className="flex items-center gap-1">
                  <span>{t('menu.language')}</span>
                  {LANGUAGE_OPTIONS.map((option) => (
                    <button
                      key={option.code}
                      onClick={() => i18n.changeLanguage(option.code)}
                      className={`rounded px-1.5 py-0.5 ${i18n.resolvedLanguage === option.code ? 'bg-[rgba(255,153,0,0.16)] text-[var(--accent)]' : 'text-[var(--text-secondary)] hover:bg-white/6'}`}
                    >
                      {option.label}
                    </button>
                  ))}
                </div>
              </div>
            </div>
          </div>

          <div className="border-b border-[var(--panel-border)] bg-[var(--chrome-bg)] px-3 py-2.5">
            <div className="flex items-center gap-3">
              <div className="flex items-center gap-1 rounded-[8px] border border-[var(--panel-border)] bg-[#191919] p-1">
                <button onClick={toggleLeftPanel} className="rounded-md px-2 py-2 text-[var(--text-secondary)] hover:bg-white/6 hover:text-[var(--text-primary)]">
                  {leftCollapsed ? <PanelLeftOpen size={15} /> : <PanelLeftClose size={15} />}
                </button>
                {TOOL_ITEMS.map(({ id, shortcut, icon: Icon }) => (
                  <button
                    key={id}
                    onClick={() => setActiveTool(id)}
                    className={`group flex items-center gap-2 rounded-md px-3 py-2 text-xs ${
                      activeTool === id
                        ? 'bg-[linear-gradient(180deg,#3e2b12,#2a2117)] text-[var(--accent)] shadow-[inset_0_0_0_1px_rgba(255,153,0,0.25)]'
                        : 'text-[var(--text-secondary)] hover:bg-white/6 hover:text-[var(--text-primary)]'
                    }`}
                    >
                    <Icon size={14} />
                    <span>{t(`tools.${id}`)}</span>
                    <span className="rounded-sm bg-black/25 px-1.5 py-0.5 font-mono text-[10px] text-[var(--text-muted)]">{shortcut}</span>
                  </button>
                ))}
                <button onClick={toggleRightPanel} className="rounded-md px-2 py-2 text-[var(--text-secondary)] hover:bg-white/6 hover:text-[var(--text-primary)]">
                  {rightCollapsed ? <PanelRightOpen size={15} /> : <PanelRightClose size={15} />}
                </button>
              </div>

              <div className="min-w-0 flex-1">
                <FindLocateBar
                  onSelect={(location) => {
                    startTransition(() => {
                      setFocusLocation(location)
                      setActiveTool('find')
                    })
                  }}
                />
              </div>

              <div className="flex items-center gap-2">
                <button onClick={() => setSettingsOpen(true)} className="desktop-ghost-button">
                  <Settings2 size={14} />
                  {t('shell.preferences')}
                </button>
                <button onClick={() => setChangelogOpen(true)} className="desktop-ghost-button">
                  <BookOpen size={14} />
                  {t('shell.releaseNotes')}
                </button>
                <button onClick={() => setOnboardingOpen(true)} className="desktop-ghost-button">
                  <CircleHelp size={14} />
                  {t('menu.help')}
                </button>
              </div>
            </div>
            <div className="mt-2 flex items-center justify-between text-[11px] text-[var(--text-muted)]">
              <div className="flex items-center gap-4 uppercase tracking-[0.18em]">
                <span>{t('shell.toolActive', { tool: t(`tools.${activeTool}`) })}</span>
                <span>{t('shell.panelsFloating', { count: Object.values(floatingPanels).filter((panel) => !panel.docked).length })}</span>
                <span>
                  {filters.militaryTypes.length + filters.trackedCategories.length + filters.yachtCategories.length > 0
                    ? t('shell.filters', { count: filters.militaryTypes.length + filters.trackedCategories.length + filters.yachtCategories.length })
                    : t('shell.filtersNoneLabel')}
                </span>
              </div>
              <div className="font-mono">{new Date().toISOString().replace('T', ' ').slice(0, 19)} UTC</div>
            </div>
          </div>

          <div className="min-h-0 flex-1">
            <Group orientation="horizontal" className="h-full">
              <Panel
                panelRef={leftPanelRef}
                defaultSize="8%"
                minSize="6%"
                maxSize="12%"
                collapsible
                collapsedSize={0}
                className="border-r border-[var(--panel-border)] bg-[var(--panel-bg)]"
              >
                <div className="flex h-full flex-col">
                  <div className="border-b border-[var(--panel-border)] bg-[var(--panel-header)] px-3 py-3">
                    <div className="text-[10px] uppercase tracking-[0.28em] text-[var(--text-muted)]">{t('menu.tools')}</div>
                  </div>
                  <div className="flex flex-1 flex-col items-center gap-2 p-2">
                    {TOOL_ITEMS.map(({ id, shortcut, icon: Icon }) => (
                      <button
                        key={id}
                        onClick={() => setActiveTool(id)}
                        className={`flex w-full flex-col items-center justify-center rounded-[10px] border px-2 py-3 text-center ${
                          activeTool === id
                            ? 'border-[rgba(255,153,0,0.35)] bg-[linear-gradient(180deg,#342513,#251d15)] text-[var(--accent)]'
                            : 'border-[var(--panel-border)] bg-[#1f1f1f] text-[var(--text-secondary)] hover:bg-[#252525] hover:text-[var(--text-primary)]'
                        }`}
                        >
                        <Icon size={18} />
                        <span className="mt-2 text-[11px] uppercase tracking-[0.12em]">{t(`tools.${id}`)}</span>
                        <span className="mt-1 rounded-sm bg-black/25 px-1.5 py-0.5 font-mono text-[10px]">{shortcut}</span>
                      </button>
                    ))}
                  </div>
                  <div className="grid gap-2 border-t border-[var(--panel-border)] p-2">
                    <button onClick={() => setInspectorTab('layers')} className="desktop-side-button">
                      <Layers3 size={14} />
                      {t('inspector.layers')}
                    </button>
                    <button onClick={() => setInspectorTab('info')} className="desktop-side-button">
                      <Info size={14} />
                      {t('mobile.intel')}
                    </button>
                  </div>
                </div>
              </Panel>

              <Separator className="desktop-resize-handle" />

              <Panel defaultSize="64%" minSize="42%">
                <div className="relative flex h-full flex-col bg-[#232323]">
                  <div className="border-b border-[var(--panel-border)] bg-[var(--panel-header)] px-4 py-2.5">
                    <div className="flex items-center justify-between">
                      <div>
                        <div className="text-[10px] uppercase tracking-[0.28em] text-[var(--text-muted)]">{t('shell.artboard')}</div>
                        <div className="mt-1 text-sm text-[var(--text-primary)]">{t('shell.mapCanvas')}</div>
                      </div>
                      <div className="flex items-center gap-2 text-[11px] uppercase tracking-[0.2em] text-[var(--text-muted)]">
                        <span>{focusLocation?.label ? t('shell.focus', { label: focusLocation.label.slice(0, 18) }) : t('shell.globalView')}</span>
                        <span className="rounded-sm border border-[var(--panel-border)] bg-[#181818] px-2 py-1 text-[var(--accent)]">{zoom.toFixed(2)}x</span>
                      </div>
                    </div>
                  </div>

                  <div className="relative min-h-0 flex-1 bg-[#2b2b2b] p-3">
                    <div className="relative h-full overflow-hidden rounded-[14px] border border-[#111] bg-[#202020] shadow-[inset_0_0_0_1px_rgba(255,255,255,0.03)]">
                      <ErrorBoundary fallbackTitle={t('shell.mapCanvas')}>
                        <MapView
                          data={dashboardData}
                          layers={layers}
                          zoom={zoom}
                          focusLocation={focusLocation}
                          onSelect={(entity) => {
                            setSelectedEntity(entity)
                            setInspectorTab('properties')
                          }}
                          onMouseMove={setMouseCoords}
                          onZoomChange={setZoom}
                          onViewChange={setViewState}
                          activeTool={activeTool}
                        />
                      </ErrorBoundary>

                      <div className="pointer-events-none absolute inset-x-0 top-0 flex justify-center pt-4">
                        <div className="pointer-events-auto rounded-full border border-[var(--panel-border)] bg-[rgba(18,18,18,0.85)] px-4 py-1.5 text-[11px] uppercase tracking-[0.22em] text-[var(--text-muted)] shadow-[0_10px_30px_rgba(0,0,0,0.35)]">
                          {activeTool === 'measure' && t('shell.modeMeasure')}
                          {activeTool === 'select' && t('shell.modeSelection')}
                          {activeTool === 'find' && t('shell.modeFind')}
                          {activeTool === 'marker' && t('shell.modeMarker')}
                          {activeTool === 'filter' && t('shell.modeFilter')}
                        </div>
                      </div>

                      <FloatingWorkspacePanel
                        title={t('shell.legend')}
                        badge={`${Object.values(layers).filter(Boolean).length}`}
                        state={floatingPanels.legend}
                        onUpdate={(next) => updateFloatingPanel('legend', next)}
                        width={266}
                      >
                        <MapLegend layers={layers} />
                      </FloatingWorkspacePanel>

                      <FloatingWorkspacePanel
                        title={t('shell.navigator')}
                        badge={viewState.zoom.toFixed(1)}
                        state={floatingPanels.navigator}
                        onUpdate={(next) => updateFloatingPanel('navigator', next)}
                        width={286}
                      >
                        <NavigatorMiniMap center={viewState} focusLocation={focusLocation} />
                      </FloatingWorkspacePanel>

                      <FloatingWorkspacePanel
                        title={t('shell.filtersPanel')}
                        state={floatingPanels.filters}
                        onUpdate={(next) => updateFloatingPanel('filters', next)}
                        width={300}
                      >
                        <FilterPanel data={dashboardData} filters={filters} onChange={setFilters} />
                      </FloatingWorkspacePanel>

                      <FloatingWorkspacePanel
                        title={t('shell.signalFeeds')}
                        badge={focusLocation ? t('shell.local') : t('shell.global')}
                        state={floatingPanels.radio}
                        onUpdate={(next) => updateFloatingPanel('radio', next)}
                        width={300}
                      >
                        <RadioInterceptPanel location={focusLocation} />
                      </FloatingWorkspacePanel>

                      <div className="absolute bottom-4 left-4 right-4 flex items-center justify-between gap-3">
                        <div className="rounded-[10px] border border-[var(--panel-border)] bg-[rgba(18,18,18,0.86)] px-3 py-2 text-[11px] uppercase tracking-[0.22em] text-[var(--text-muted)] shadow-[0_14px_36px_rgba(0,0,0,0.32)]">
                          {t('shell.dockedPanels')}
                          <div className="mt-2 flex flex-wrap gap-2">
                            {(Object.entries(floatingPanels) as Array<[FloatingPanelId, FloatingPanelState]>)
                              .filter(([, panel]) => panel.docked)
                              .map(([id, panel]) => (
                                <button
                                  key={id}
                                  onClick={() => updateFloatingPanel(id, { ...panel, docked: false })}
                                  className="rounded-md border border-[var(--panel-border)] bg-[#222] px-2 py-1 text-[10px] uppercase tracking-[0.18em] text-[var(--text-secondary)] hover:text-[var(--text-primary)]"
                                >
                                  {id}
                                </button>
                              ))}
                          </div>
                        </div>

                        <div className="rounded-[10px] border border-[var(--panel-border)] bg-[rgba(18,18,18,0.86)] px-3 py-2 text-[11px] text-[var(--text-secondary)] shadow-[0_14px_36px_rgba(0,0,0,0.32)]">
                          <div className="font-mono uppercase tracking-[0.2em] text-[var(--text-muted)]">
                            {viewState.lat.toFixed(2)} / {viewState.lng.toFixed(2)}
                          </div>
                          <div className="mt-1 text-xs text-[var(--text-primary)]">{focusLocation?.label || t('shell.noPinnedTarget')}</div>
                        </div>
                      </div>
                    </div>
                  </div>
                </div>
              </Panel>

              <Separator className="desktop-resize-handle" />

              <Panel
                panelRef={rightPanelRef}
                defaultSize="28%"
                minSize="20%"
                maxSize="36%"
                collapsible
                collapsedSize={0}
                className="border-l border-[var(--panel-border)] bg-[var(--panel-bg)]"
              >
                <PropertiesSidebar
                  tab={inspectorTab}
                  onTabChange={setInspectorTab}
                  layers={layers}
                  counts={counts}
                  onToggleLayer={(key) => setLayers((prev) => ({ ...prev, [key]: !prev[key] }))}
                  selectedEntity={deferredSelectedEntity}
                  fastData={filteredFastData}
                  slowData={slowData}
                  focusLocation={focusLocation}
                />
              </Panel>
            </Group>
          </div>

          <StatusBar coords={mouseCoords} zoom={zoom} counts={counts} spaceWeather={slowData?.space_weather} statusLabel={statusLabel} />
        </div>

        <AnimatePresence>
          {menuOpen && (
            <motion.div
              initial={{ opacity: 0, y: -10 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0, y: -10 }}
              className="absolute inset-x-4 top-18 z-40 rounded-[12px] border border-[var(--panel-border)] bg-[var(--panel-bg)] p-4 shadow-[0_30px_80px_rgba(0,0,0,0.45)] md:hidden"
            >
              <div className="grid gap-2">
                <button onClick={() => setOnboardingOpen(true)} className="desktop-ghost-button justify-center">
                  <CircleHelp size={14} />
                  {t('menu.help')}
                </button>
                <button onClick={() => setChangelogOpen(true)} className="desktop-ghost-button justify-center">
                  <BookOpen size={14} />
                  {t('shell.releaseNotes')}
                </button>
                <button onClick={() => setSettingsOpen(true)} className="desktop-ghost-button justify-center">
                  <Settings2 size={14} />
                  {t('shell.preferences')}
                </button>
              </div>
            </motion.div>
          )}
        </AnimatePresence>

        <div className="absolute inset-x-0 top-0 z-40 border-b border-[var(--panel-border)] bg-[var(--chrome-bg)] px-4 py-3 md:hidden">
          <div className="flex items-center justify-between">
            <div>
              <div className="text-[10px] uppercase tracking-[0.28em] text-[var(--text-muted)]">{t('app.title')}</div>
              <div className="text-lg font-semibold text-[var(--text-primary)]">{t('app.desktopUi')}</div>
            </div>
            <button onClick={() => setMenuOpen((value) => !value)} className="rounded-md border border-[var(--panel-border)] bg-[#1f1f1f] p-2 text-[var(--text-secondary)]">
              {menuOpen ? <X size={16} /> : <Menu size={16} />}
            </button>
          </div>
        </div>

        <div className="md:hidden pt-[72px]">
          <MobileDashboard
            layers={layers}
            counts={counts}
            onToggleLayer={(key) => setLayers((prev) => ({ ...prev, [key]: !prev[key] }))}
            data={dashboardData}
            filters={filters}
            onFiltersChange={setFilters}
            focusLocation={focusLocation}
            onSearchSelect={(location) => {
              startTransition(() => {
                setFocusLocation(location)
                setActiveTool('find')
              })
            }}
            onSelectEntity={setSelectedEntity}
            onMouseMove={setMouseCoords}
            zoom={zoom}
            onZoomChange={setZoom}
            activeTool={activeTool}
            selectedEntity={deferredSelectedEntity}
          />
        </div>

        <SettingsPanel open={settingsOpen} onClose={() => setSettingsOpen(false)} />
        <OnboardingModal open={onboardingOpen} onClose={() => setOnboardingOpen(false)} />
        <ChangelogModal open={changelogOpen} onClose={() => setChangelogOpen(false)} />
      </div>
    </DashboardDataProvider>
  )
}
