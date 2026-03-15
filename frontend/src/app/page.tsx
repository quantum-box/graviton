'use client'

import type { ReactNode } from 'react'
import { startTransition, useDeferredValue, useEffect, useMemo, useState } from 'react'
import dynamic from 'next/dynamic'
import { AnimatePresence, motion } from 'framer-motion'
import {
  BookOpen,
  Bell,
  CircleHelp,
  Crosshair,
  Filter,
  Layers3,
  Lock,
  Map,
  Radio,
  Search,
  ShieldAlert,
  Settings2,
  Sparkles,
  X,
} from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { FilterPanel, type FilterState } from '@/components/FilterPanel'
import { FindLocateBar } from '@/components/FindLocateBar'
import { LeftPanel } from '@/components/LeftPanel'
import { MapLegend } from '@/components/MapLegend'
import { OnboardingModal } from '@/components/OnboardingModal'
import { RightPanel } from '@/components/RightPanel'
import { SettingsPanel } from '@/components/SettingsPanel'
import { StatusBar } from '@/components/StatusBar'
import { ChangelogModal } from '@/components/ChangelogModal'
import { ErrorBoundary } from '@/components/ErrorBoundary'
import { RadioInterceptPanel } from '@/components/RadioInterceptPanel'
import { AuthPanel } from '@/components/AuthPanel'
import { AlertTray } from '@/components/AlertTray'
import { C2Panel } from '@/components/C2Panel'
import { useDataPolling } from '@/hooks/useDataPolling'
import { useWebSocket } from '@/hooks/useWebSocket'
import { DashboardDataProvider } from '@/lib/DashboardDataContext'

const MapView = dynamic(() => import('@/components/MapView'), { ssr: false })

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
  fused_objects: boolean
  simulation_markers: boolean
  shared_objects: boolean
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
  sentinel_overlay: boolean
  trajectories: boolean
  predictions: boolean
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
  fused_objects: true,
  simulation_markers: true,
  shared_objects: true,
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
  sentinel_overlay: false,
  trajectories: true,
  predictions: true,
  day_night: true,
}

const DEFAULT_FILTERS: FilterState = {
  militaryTypes: [],
  trackedCategories: [],
  yachtCategories: [],
}

type FlyoutId = 'layers' | 'filters' | 'intel'
type MobileSheetId = FlyoutId | 'search' | 'alerts'

const DESKTOP_FLYOUTS: Array<{ id: FlyoutId; label: string; icon: typeof Layers3 }> = [
  { id: 'layers', label: 'Sources', icon: Layers3 },
  { id: 'filters', label: 'Filters', icon: Filter },
  { id: 'intel', label: 'Signals', icon: Radio },
]

const MOBILE_NAV_ITEMS: Array<{ id: 'map' | MobileSheetId | 'settings'; label: string; icon: typeof Layers3 }> = [
  { id: 'map', label: 'Map', icon: Map as typeof Layers3 },
  { id: 'layers', label: 'Layers', icon: Layers3 },
  { id: 'search', label: 'Search', icon: Search as typeof Layers3 },
  { id: 'alerts', label: 'Alerts', icon: Bell as typeof Layers3 },
  { id: 'settings', label: 'Settings', icon: Settings2 as typeof Layers3 },
]

function TopToolbar({
  onSearchSelect,
  isLoading,
  onOpenSettings,
  onOpenAuth,
  onOpenOnboarding,
  onOpenChangelog,
}: {
  onSearchSelect: (result: { lat: number; lng: number; label?: string } | null) => void
  isLoading: boolean
  onOpenSettings: () => void
  onOpenAuth: () => void
  onOpenOnboarding: () => void
  onOpenChangelog: () => void
}) {
  return (
    <div className="pointer-events-auto flex h-10 items-center gap-3 rounded-full border border-white/12 bg-[rgba(7,14,24,0.82)] px-3 shadow-[0_18px_48px_rgba(0,0,0,0.32)] backdrop-blur-xl">
      <div className="flex items-center gap-2 pr-1">
        <div className="flex h-7 w-7 items-center justify-center rounded-full bg-[linear-gradient(135deg,#2dd4bf,#0f766e)] text-slate-950">
          <Map size={15} />
        </div>
        <div className="text-[11px] font-semibold uppercase tracking-[0.28em] text-white">GRAVITON</div>
      </div>

      <div className="hidden min-w-[320px] flex-1 md:block">
        <FindLocateBar onSelect={(location) => onSearchSelect(location)} />
      </div>

      <div className="ml-auto flex items-center gap-1.5">
        <button
          type="button"
          onClick={onOpenOnboarding}
          className="hidden h-8 items-center justify-center rounded-full border border-white/10 px-3 text-xs text-slate-300 transition hover:bg-white/8 lg:inline-flex"
          aria-label="Open guide"
        >
          <Sparkles size={14} />
        </button>
        <button
          type="button"
          onClick={onOpenChangelog}
          className="hidden h-8 items-center justify-center rounded-full border border-white/10 px-3 text-xs text-slate-300 transition hover:bg-white/8 lg:inline-flex"
          aria-label="Open release notes"
        >
          <BookOpen size={14} />
        </button>
        <button
          type="button"
          onClick={onOpenAuth}
          className="hidden h-8 items-center justify-center rounded-full border border-white/10 px-3 text-xs text-slate-300 transition hover:bg-white/8 lg:inline-flex"
          aria-label="Open auth"
        >
          <Lock size={14} />
        </button>
        <button
          type="button"
          onClick={onOpenSettings}
          className="flex h-8 w-8 items-center justify-center rounded-full border border-white/10 text-slate-300 transition hover:bg-white/8"
          aria-label="Open settings"
        >
          <Settings2 size={15} />
        </button>
        <div className="hidden min-w-[60px] text-right text-[10px] font-medium uppercase tracking-[0.2em] text-teal-200 md:block">
          {isLoading ? 'Syncing' : 'Live'}
        </div>
      </div>
    </div>
  )
}

function SidebarButton({
  label,
  active,
  onClick,
  icon: Icon,
}: {
  label: string
  active: boolean
  onClick: () => void
  icon: typeof Layers3
}) {
  return (
    <div className="group relative">
      <button
        type="button"
        onClick={onClick}
        className={`flex h-10 w-10 items-center justify-center rounded-2xl border transition ${
          active
            ? 'border-teal-300/40 bg-teal-400/18 text-teal-100'
            : 'border-white/10 bg-[rgba(9,14,22,0.84)] text-slate-300 hover:bg-white/10'
        }`}
        title={label}
        aria-label={label}
      >
        <Icon size={18} />
      </button>
      <div className="pointer-events-none absolute left-[calc(100%+10px)] top-1/2 hidden -translate-y-1/2 whitespace-nowrap rounded-full border border-white/10 bg-[rgba(7,14,24,0.92)] px-2.5 py-1 text-[11px] text-white shadow-lg group-hover:block">
        {label}
      </div>
    </div>
  )
}

function FlyoutShell({
  title,
  onClose,
  children,
  className = '',
}: {
  title: string
  onClose: () => void
  children: ReactNode
  className?: string
}) {
  return (
    <motion.section
      initial={{ opacity: 0, x: -12 }}
      animate={{ opacity: 1, x: 0 }}
      exit={{ opacity: 0, x: -12 }}
      transition={{ duration: 0.18, ease: 'easeOut' }}
      className={`pointer-events-auto flex max-h-[min(720px,calc(100vh-120px))] w-[min(360px,calc(100vw-88px))] flex-col overflow-hidden rounded-[26px] border border-white/10 bg-[rgba(7,14,24,0.9)] shadow-[0_24px_72px_rgba(0,0,0,0.42)] backdrop-blur-xl ${className}`}
    >
      <div className="flex items-center justify-between border-b border-white/8 px-4 py-3">
        <div className="text-[11px] font-semibold uppercase tracking-[0.26em] text-slate-300">{title}</div>
        <button
          type="button"
          onClick={onClose}
          className="flex h-8 w-8 items-center justify-center rounded-full border border-white/10 text-slate-300 transition hover:bg-white/8"
          aria-label={`Close ${title}`}
        >
          <X size={15} />
        </button>
      </div>
      <div className="min-h-0 flex-1 overflow-y-auto p-3">{children}</div>
    </motion.section>
  )
}

function MobileBottomSheet({
  title,
  onClose,
  children,
}: {
  title: string
  onClose: () => void
  children: ReactNode
}) {
  const [dragOffset, setDragOffset] = useState(0)
  const [touchStartY, setTouchStartY] = useState<number | null>(null)

  function handleTouchStart(event: React.TouchEvent<HTMLDivElement>) {
    setTouchStartY(event.touches[0]?.clientY ?? null)
  }

  function handleTouchMove(event: React.TouchEvent<HTMLDivElement>) {
    if (touchStartY === null) return
    const nextOffset = Math.max(0, (event.touches[0]?.clientY ?? touchStartY) - touchStartY)
    setDragOffset(nextOffset)
  }

  function handleTouchEnd() {
    if (dragOffset > 84) {
      onClose()
    }
    setDragOffset(0)
    setTouchStartY(null)
  }

  return (
    <motion.div
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      exit={{ opacity: 0 }}
      className="pointer-events-auto absolute inset-0 z-50 bg-black/35 md:hidden"
      onClick={onClose}
    >
      <motion.section
        initial={{ y: '100%' }}
        animate={{ y: dragOffset }}
        exit={{ y: '100%' }}
        transition={{ duration: 0.22, ease: 'easeOut' }}
        className="absolute inset-x-0 bottom-[calc(var(--bottom-nav-height)+env(safe-area-inset-bottom)+4px)] max-h-[min(78vh,calc(100dvh-var(--bottom-nav-height)-env(safe-area-inset-bottom)-48px))] overflow-hidden rounded-t-[28px] border-t border-white/10 bg-[rgba(7,14,24,0.96)] shadow-[0_-18px_60px_rgba(0,0,0,0.38)] backdrop-blur-xl"
        onClick={(event) => event.stopPropagation()}
      >
        <div
          className="flex items-center justify-between px-4 py-3"
          onTouchStart={handleTouchStart}
          onTouchMove={handleTouchMove}
          onTouchEnd={handleTouchEnd}
        >
          <div>
            <div className="mx-auto mb-2 h-1.5 w-12 rounded-full bg-white/15" />
            <div className="text-[11px] font-semibold uppercase tracking-[0.26em] text-slate-300">{title}</div>
          </div>
          <button
            type="button"
            onClick={onClose}
            className="flex h-8 w-8 items-center justify-center rounded-full border border-white/10 text-slate-300"
            aria-label={`Close ${title}`}
          >
            <X size={15} />
          </button>
        </div>
        <div className="max-h-[calc(min(78vh,calc(100dvh-var(--bottom-nav-height)-env(safe-area-inset-bottom)-48px))-64px)] overflow-y-auto px-4 pb-6">{children}</div>
      </motion.section>
    </motion.div>
  )
}

export default function Dashboard() {
  const { t } = useTranslation()
  const [settingsOpen, setSettingsOpen] = useState(false)
  const [authOpen, setAuthOpen] = useState(false)
  const [onboardingOpen, setOnboardingOpen] = useState(false)
  const [changelogOpen, setChangelogOpen] = useState(false)
  const [layers, setLayers] = useState<LayerVisibility>(DEFAULT_LAYERS)
  const [selectedEntity, setSelectedEntity] = useState<any>(null)
  const [mouseCoords, setMouseCoords] = useState<{ lat: number; lng: number } | null>(null)
  const [zoom, setZoom] = useState(2.8)
  const [focusLocation, setFocusLocation] = useState<{ lat: number; lng: number; label?: string } | null>(null)
  const [filters, setFilters] = useState<FilterState>(DEFAULT_FILTERS)
  const [leftFlyout, setLeftFlyout] = useState<FlyoutId | null>('layers')
  const [mobileSheet, setMobileSheet] = useState<MobileSheetId | null>(null)
  const [rightPanelOpen, setRightPanelOpen] = useState(false)
  const [lastMobileSheet, setLastMobileSheet] = useState<MobileSheetId>('layers')
  const [mobileSwipeStartY, setMobileSwipeStartY] = useState<number | null>(null)
  const [authToken, setAuthToken] = useState<string | null>(null)
  const [authUser, setAuthUser] = useState<any>(null)
  const [sharedObjects, setSharedObjects] = useState<any[]>([])
  const [trajectory, setTrajectory] = useState<Array<{ lat: number; lng: number }>>([])
  const [prediction, setPrediction] = useState<Array<{ lat: number; lng: number }>>([])
  const [c2State, setC2State] = useState<{ watchlist: any[]; missions: any[]; alerts: any[] } | null>(null)
  const [layout, setLayout] = useState({ c2: { x: 20, y: 420 } })
  const deferredSelectedEntity = useDeferredValue(selectedEntity)
  const { fastData, slowData, isLoading } = useDataPolling()
  const liveSocket = useWebSocket()

  const mergedFastData = useMemo(() => {
    if (!fastData) return null
    return {
      ...fastData,
      fused_objects: liveSocket.datasets.fused_objects ?? fastData.fused_objects ?? [],
      simulation_markers: liveSocket.datasets.simulation_markers ?? fastData.simulation_markers ?? [],
      alerts: liveSocket.alerts.length > 0 ? liveSocket.alerts : fastData.alerts ?? [],
    }
  }, [fastData, liveSocket.alerts, liveSocket.datasets])

  const mergedSlowData = useMemo(() => {
    if (!slowData) return null
    return {
      ...slowData,
      shared_objects: sharedObjects.flatMap((item) => {
        const payload = item.payload ?? {}
        if (typeof payload.lat !== 'number' || typeof payload.lng !== 'number') return []
        return [{ ...payload, id: item.id, title: item.title }]
      }),
    }
  }, [slowData, sharedObjects])

  const filteredFastData = useMemo(() => {
    if (!mergedFastData) return null
    return {
      ...mergedFastData,
      military_flights:
        filters.militaryTypes.length === 0
          ? mergedFastData.military_flights
          : mergedFastData.military_flights.filter((item: any) => filters.militaryTypes.includes(item.military_type)),
      tracked_flights:
        filters.trackedCategories.length === 0
          ? mergedFastData.tracked_flights
          : mergedFastData.tracked_flights.filter((item: any) => filters.trackedCategories.includes(item.alert_category)),
      ships:
        filters.yachtCategories.length === 0
          ? mergedFastData.ships
          : mergedFastData.ships.filter((item: any) => filters.yachtCategories.includes(item.yacht_category)),
    }
  }, [mergedFastData, filters])

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
      fused_objects: filteredFastData?.fused_objects?.length ?? 0,
      simulation_markers: filteredFastData?.simulation_markers?.length ?? 0,
      shared_objects: mergedSlowData?.shared_objects?.length ?? 0,
      earthquakes: mergedSlowData?.earthquakes?.length ?? 0,
      news: mergedSlowData?.news?.length ?? 0,
      firms_fires: mergedSlowData?.firms_fires?.length ?? 0,
      gdelt: mergedSlowData?.gdelt?.length ?? 0,
      liveuamap: mergedSlowData?.liveuamap?.length ?? 0,
      frontlines: mergedSlowData?.frontlines?.features?.length ?? 0,
      internet_outages: mergedSlowData?.internet_outages?.length ?? 0,
      kiwisdr: mergedSlowData?.kiwisdr?.length ?? 0,
      datacenters: mergedSlowData?.datacenters?.length ?? 0,
      cctv: mergedSlowData?.cctv?.length ?? 0,
      sentinel_overlay: 1,
      trajectories: trajectory.length,
      predictions: prediction.length,
    }),
    [filteredFastData, mergedSlowData, trajectory.length, prediction.length],
  )

  const dashboardData = { fastData: filteredFastData, slowData: mergedSlowData }

  useEffect(() => {
    const seen = window.localStorage.getItem('graviton-onboarding-seen')
    if (!seen) {
      setOnboardingOpen(true)
      window.localStorage.setItem('graviton-onboarding-seen', 'true')
    }
    const savedToken = window.localStorage.getItem('graviton-jwt')
    const savedUser = window.localStorage.getItem('graviton-user')
    const savedLayout = window.localStorage.getItem('graviton-dashboard-layout')
    const savedLayers = window.localStorage.getItem('graviton-layer-visibility')
    if (savedToken) setAuthToken(savedToken)
    if (savedUser) setAuthUser(JSON.parse(savedUser))
    if (savedLayout) setLayout(JSON.parse(savedLayout))
    if (savedLayers) setLayers(JSON.parse(savedLayers))
  }, [])

  useEffect(() => {
    if (selectedEntity) setRightPanelOpen(true)
  }, [selectedEntity])

  useEffect(() => {
    if (mobileSheet) setLastMobileSheet(mobileSheet)
  }, [mobileSheet])

  useEffect(() => {
    window.localStorage.setItem('graviton-dashboard-layout', JSON.stringify(layout))
  }, [layout])

  useEffect(() => {
    window.localStorage.setItem('graviton-layer-visibility', JSON.stringify(layers))
  }, [layers])

  useEffect(() => {
    if (!authToken) return
    fetch('/api/auth/me', { headers: { Authorization: `Bearer ${authToken}` } })
      .then((resp) => (resp.ok ? resp.json() : null))
      .then((data) => {
        if (data?.user) {
          setAuthUser(data.user)
          window.localStorage.setItem('graviton-jwt', authToken)
          window.localStorage.setItem('graviton-user', JSON.stringify(data.user))
        }
      })
      .catch(() => {})
  }, [authToken])

  useEffect(() => {
    if (!authToken) return
    fetch('/api/team/shared', { headers: { Authorization: `Bearer ${authToken}` } })
      .then((resp) => (resp.ok ? resp.json() : []))
      .then((data) => setSharedObjects(Array.isArray(data) ? data : []))
      .catch(() => setSharedObjects([]))
  }, [authToken])

  useEffect(() => {
    fetch('/api/c2/state')
      .then((resp) => resp.json())
      .then((data) => setC2State(data))
      .catch(() => setC2State({ watchlist: [], missions: [], alerts: [] }))
  }, [liveSocket.alerts.length])

  useEffect(() => {
    const objectId = selectedEntity?.icao24 ?? selectedEntity?.mmsi ?? selectedEntity?.id
    const objectType = selectedEntity?.mmsi ? 'ship' : selectedEntity?.icao24 ? 'aircraft' : null
    if (!objectId || !objectType) {
      setTrajectory([])
      setPrediction([])
      return
    }
    fetch(`/api/analysis/trajectories?object_type=${objectType}&object_id=${objectId}`)
      .then((resp) => resp.json())
      .then((data) => setTrajectory(data.points ?? []))
      .catch(() => setTrajectory([]))
    fetch(`/api/analysis/predictions?object_type=${objectType}&object_id=${objectId}`)
      .then((resp) => resp.json())
      .then((data) => setPrediction(data.line ?? []))
      .catch(() => setPrediction([]))
  }, [selectedEntity])

  function renderFlyoutContent(id: FlyoutId) {
    if (id === 'layers') {
      return (
        <div className="space-y-3">
          <LeftPanel
            layers={layers}
            counts={counts}
            onToggle={(key) => setLayers((prev) => ({ ...prev, [key]: !prev[key] }))}
            className="border-r-0 bg-transparent"
          />
          <MapLegend layers={layers} />
        </div>
      )
    }

    if (id === 'filters') {
      return <FilterPanel data={dashboardData} filters={filters} onChange={setFilters} />
    }

    return <RadioInterceptPanel location={focusLocation} />
  }

  function renderMobileSheetContent(id: MobileSheetId) {
    if (id === 'search') {
      return (
        <div className="space-y-4">
          <FindLocateBar
            onSelect={(location) => {
              setFocusLocation(location)
              setMobileSheet(null)
            }}
          />
          <div className="rounded-3xl border border-white/10 bg-white/5 p-4 text-sm text-slate-300">
            Search for a city, port, base, or paste coordinates as `lat, lng`.
          </div>
        </div>
      )
    }

    if (id === 'alerts') {
      const alerts = filteredFastData?.alerts ?? liveSocket.alerts
      if (alerts.length === 0) {
        return <div className="rounded-3xl border border-white/10 bg-white/5 p-4 text-sm text-slate-400">No active alerts.</div>
      }

      return (
        <div className="space-y-3">
          {alerts.map((alert, index) => (
            <div key={`${alert.id ?? index}`} className="rounded-3xl border border-amber-300/20 bg-[rgba(31,18,10,0.86)] p-4 shadow-lg">
              <div className="flex items-center justify-between gap-2">
                <div className="text-[11px] uppercase tracking-[0.18em] text-amber-200">{alert.severity ?? 'info'}</div>
                <div className="text-[10px] text-slate-400">{alert.source}</div>
              </div>
              <div className="mt-2 text-sm text-white">{alert.title}</div>
              <div className="mt-1 text-xs text-slate-300">{alert.message}</div>
            </div>
          ))}
        </div>
      )
    }

    return renderFlyoutContent(id)
  }

  async function handleLocateUser() {
    if (!navigator.geolocation) return

    navigator.geolocation.getCurrentPosition(
      (position) => {
        startTransition(() => {
          setFocusLocation({
            lat: position.coords.latitude,
            lng: position.coords.longitude,
            label: 'Current location',
          })
        })
      },
      () => {},
      { enableHighAccuracy: true, timeout: 10000, maximumAge: 60000 },
    )
  }

  return (
    <DashboardDataProvider fastData={filteredFastData} slowData={mergedSlowData} selectedEntity={selectedEntity} setSelectedEntity={setSelectedEntity}>
      <div
        className="fixed inset-0 h-[100dvh] w-screen overflow-hidden bg-[#07101a] text-[var(--text-primary)]"
        onTouchStart={(event) => {
          const startY = event.touches[0]?.clientY ?? 0
          if (startY > window.innerHeight - 96 && !mobileSheet && !rightPanelOpen) {
            setMobileSwipeStartY(startY)
          }
        }}
        onTouchEnd={(event) => {
          if (mobileSwipeStartY === null || mobileSheet || rightPanelOpen) {
            setMobileSwipeStartY(null)
            return
          }
          const endY = event.changedTouches[0]?.clientY ?? 0
          if (mobileSwipeStartY - endY > 72) {
            setMobileSheet(lastMobileSheet)
          }
          setMobileSwipeStartY(null)
        }}
      >
        <ErrorBoundary fallbackTitle={t('shell.mapCanvas')}>
          <MapView
            data={dashboardData}
            layers={layers}
            zoom={zoom}
            focusLocation={focusLocation}
            onSelect={setSelectedEntity}
            onMouseMove={setMouseCoords}
            onZoomChange={setZoom}
            trajectory={trajectory}
            prediction={prediction}
          />
        </ErrorBoundary>

        <div className="pointer-events-none absolute inset-0 z-20">
          <AlertTray alerts={filteredFastData?.alerts ?? liveSocket.alerts} />

          <div className="absolute inset-x-3 top-3 hidden md:block">
            <TopToolbar
              onSearchSelect={(location) => {
                startTransition(() => {
                  setFocusLocation(location)
                })
              }}
              isLoading={isLoading}
              onOpenSettings={() => setSettingsOpen(true)}
              onOpenAuth={() => setAuthOpen(true)}
              onOpenOnboarding={() => setOnboardingOpen(true)}
              onOpenChangelog={() => setChangelogOpen(true)}
            />
          </div>

          <div className="absolute left-3 top-16 bottom-16 hidden md:flex">
            <div className="pointer-events-auto flex w-12 flex-col items-center gap-2 rounded-[26px] border border-white/10 bg-[rgba(7,14,24,0.82)] px-1.5 py-2 shadow-[0_20px_50px_rgba(0,0,0,0.35)] backdrop-blur-xl">
              {DESKTOP_FLYOUTS.map(({ id, label, icon }) => (
                <SidebarButton key={id} label={label} active={leftFlyout === id} onClick={() => setLeftFlyout((current) => (current === id ? null : id))} icon={icon} />
              ))}
              <div className="mt-auto space-y-2">
                <SidebarButton label="Guide" active={false} onClick={() => setOnboardingOpen(true)} icon={CircleHelp as typeof Layers3} />
                <SidebarButton label="Notes" active={false} onClick={() => setChangelogOpen(true)} icon={BookOpen as typeof Layers3} />
              </div>
            </div>

            <AnimatePresence initial={false}>
              {leftFlyout && (
                <div className="ml-3">
                  <FlyoutShell title={DESKTOP_FLYOUTS.find((item) => item.id === leftFlyout)?.label ?? ''} onClose={() => setLeftFlyout(null)}>
                    {renderFlyoutContent(leftFlyout)}
                  </FlyoutShell>
                </div>
              )}
            </AnimatePresence>
          </div>

          <AnimatePresence initial={false}>
            {rightPanelOpen && (
              <motion.aside
                initial={{ x: 360, opacity: 0 }}
                animate={{ x: 0, opacity: 1 }}
                exit={{ x: 360, opacity: 0 }}
                transition={{ duration: 0.2, ease: 'easeOut' }}
                className="pointer-events-auto absolute right-3 top-16 bottom-16 hidden w-[min(380px,calc(100vw-32px))] overflow-hidden rounded-[28px] border border-white/10 bg-[rgba(7,14,24,0.92)] shadow-[0_24px_72px_rgba(0,0,0,0.42)] backdrop-blur-xl md:flex md:flex-col"
              >
                <div className="flex items-center justify-between border-b border-white/8 px-4 py-3">
                  <div>
                    <div className="text-[11px] font-semibold uppercase tracking-[0.26em] text-slate-300">Selection</div>
                    <div className="mt-1 text-sm text-white">{deferredSelectedEntity?.title || deferredSelectedEntity?.name || deferredSelectedEntity?.callsign || 'Object details'}</div>
                  </div>
                  <button
                    type="button"
                    onClick={() => setRightPanelOpen(false)}
                    className="flex h-8 w-8 items-center justify-center rounded-full border border-white/10 text-slate-300 transition hover:bg-white/8"
                    aria-label="Close selection panel"
                  >
                    <X size={15} />
                  </button>
                </div>
                <div className="min-h-0 flex-1 overflow-y-auto">
                  <RightPanel selectedEntity={deferredSelectedEntity} fastData={filteredFastData} slowData={mergedSlowData} focusLocation={focusLocation} className="h-full bg-transparent" />
                </div>
              </motion.aside>
            )}
          </AnimatePresence>

          <div className="absolute inset-x-2 top-2 md:hidden">
            <div className="pointer-events-auto flex h-8 items-center gap-2 rounded-full border border-white/12 bg-[rgba(7,14,24,0.74)] px-3 shadow-[0_18px_48px_rgba(0,0,0,0.32)] backdrop-blur-xl">
              <div className="flex items-center gap-2 text-[11px] font-semibold uppercase tracking-[0.22em] text-white">
                <ShieldAlert size={13} />
                <span>GRAVITON</span>
              </div>
              <div className="ml-auto text-[10px] uppercase tracking-[0.2em] text-teal-200">{isLoading ? 'Sync' : 'Live'}</div>
              <button
                type="button"
                onClick={() => setAuthOpen(true)}
                className="flex h-6 w-6 items-center justify-center rounded-full border border-white/10 text-slate-300"
                aria-label="Open auth"
              >
                <Lock size={12} />
              </button>
              <button
                type="button"
                onClick={() => setSettingsOpen(true)}
                className="flex h-6 w-6 items-center justify-center rounded-full border border-white/10 text-slate-300"
                aria-label="Open settings"
              >
                <Settings2 size={12} />
              </button>
            </div>
          </div>

          <button
            type="button"
            onClick={() => void handleLocateUser()}
            className="pointer-events-auto absolute bottom-[calc(var(--bottom-nav-height)+env(safe-area-inset-bottom)+14px)] right-3 z-30 flex h-12 w-12 items-center justify-center rounded-full bg-[linear-gradient(135deg,#2dd4bf,#0f766e)] text-slate-950 shadow-[0_16px_40px_rgba(15,118,110,0.45)] md:hidden"
            aria-label="Move to current location"
          >
            <Crosshair size={18} />
          </button>

          <div className="absolute inset-x-0 bottom-0 md:hidden">
            <div className="pointer-events-auto flex h-[calc(var(--bottom-nav-height)+env(safe-area-inset-bottom))] items-start justify-around border-t border-white/10 bg-[rgba(7,14,24,0.94)] px-2 pt-2 shadow-[0_-18px_48px_rgba(0,0,0,0.32)] backdrop-blur-xl">
              {MOBILE_NAV_ITEMS.map(({ id, label, icon: Icon }) => {
                const active =
                  (id === 'map' && !mobileSheet && !rightPanelOpen) ||
                  (id !== 'map' && id !== 'settings' && mobileSheet === id) ||
                  (id === 'settings' && settingsOpen)

                return (
                  <button
                    key={id}
                    type="button"
                    onClick={() => {
                      if (id === 'map') {
                        setMobileSheet(null)
                        setRightPanelOpen(false)
                        return
                      }

                      if (id === 'settings') {
                        setSettingsOpen(true)
                        return
                      }

                      setRightPanelOpen(false)
                      setMobileSheet(id)
                    }}
                    className={`flex min-w-0 flex-1 flex-col items-center gap-1 rounded-2xl px-1 py-1.5 text-[10px] uppercase tracking-[0.18em] transition ${
                      active ? 'text-teal-200' : 'text-slate-400'
                    }`}
                    aria-label={label}
                  >
                    <div className={`flex h-8 w-8 items-center justify-center rounded-full border ${active ? 'border-teal-300/30 bg-teal-400/15' : 'border-white/10 bg-white/5'}`}>
                      <Icon size={16} />
                    </div>
                    <span className="truncate">{label}</span>
                  </button>
                )
              })}
            </div>
          </div>

          <AnimatePresence initial={false}>
            {mobileSheet && (
              <MobileBottomSheet
                title={mobileSheet === 'search' ? 'Search' : mobileSheet === 'alerts' ? 'Alerts' : DESKTOP_FLYOUTS.find((item) => item.id === mobileSheet)?.label ?? ''}
                onClose={() => setMobileSheet(null)}
              >
                {renderMobileSheetContent(mobileSheet)}
              </MobileBottomSheet>
            )}
          </AnimatePresence>

          <AnimatePresence initial={false}>
            {rightPanelOpen && deferredSelectedEntity && (
              <div className="md:hidden"
              >
                <MobileBottomSheet title="Selection" onClose={() => setRightPanelOpen(false)}>
                  <RightPanel selectedEntity={deferredSelectedEntity} fastData={filteredFastData} slowData={mergedSlowData} focusLocation={focusLocation} className="bg-transparent" />
                </MobileBottomSheet>
              </div>
            )}
          </AnimatePresence>
        </div>

        <div className="hidden md:block">
          <C2Panel
            token={authToken}
            state={c2State}
            selectedEntity={deferredSelectedEntity}
            onRefresh={() => {
              fetch('/api/c2/state')
                .then((resp) => resp.json())
                .then((data) => setC2State(data))
                .catch(() => {})
            }}
            position={layout.c2}
            onMove={(position) => setLayout((current) => ({ ...current, c2: position }))}
          />
        </div>

        <div className="pointer-events-none absolute inset-x-3 bottom-[calc(var(--bottom-nav-height)+env(safe-area-inset-bottom)+8px)] z-20 md:bottom-3">
          <StatusBar coords={mouseCoords} zoom={zoom} counts={counts} statusLabel={focusLocation?.label} />
        </div>

        <SettingsPanel open={settingsOpen} onClose={() => setSettingsOpen(false)} token={authToken} />
        <AuthPanel
          token={authToken}
          user={authUser}
          open={authOpen}
          onClose={() => setAuthOpen(false)}
          onAuthenticated={({ token, user }) => {
            setAuthToken(token)
            setAuthUser(user)
            window.localStorage.setItem('graviton-jwt', token)
            window.localStorage.setItem('graviton-user', JSON.stringify(user))
          }}
        />
        <OnboardingModal open={onboardingOpen} onClose={() => setOnboardingOpen(false)} />
        <ChangelogModal open={changelogOpen} onClose={() => setChangelogOpen(false)} />
      </div>
    </DashboardDataProvider>
  )
}
