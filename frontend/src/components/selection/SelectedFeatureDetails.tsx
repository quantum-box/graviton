'use client'

import { PanelCard } from '@/components/panels/common'
import { getFlightOperator } from '@/utils/airlineCodes'

type DetailRow = {
  label: string
  value: string
}

type DetailPayload = {
  eyebrow: string
  title: string
  subtitle?: string | null
  summary?: string | null
  fields: DetailRow[]
  link?: string | null
  linkLabel?: string | null
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null
}

function toText(value: unknown): string | null {
  if (typeof value === 'string') {
    const normalized = value.replace(/<[^>]+>/g, ' ').replace(/\s+/g, ' ').trim()
    return normalized || null
  }
  if (typeof value === 'number' && Number.isFinite(value)) {
    return String(value)
  }
  if (typeof value === 'boolean') {
    return value ? 'Yes' : 'No'
  }
  return null
}

function toNumber(value: unknown): number | null {
  if (typeof value === 'number' && Number.isFinite(value)) return value
  if (typeof value === 'string') {
    const parsed = Number(value)
    if (Number.isFinite(parsed)) return parsed
  }
  return null
}

function firstText(...values: unknown[]) {
  for (const value of values) {
    const normalized = toText(value)
    if (normalized) return normalized
  }
  return null
}

function firstNumber(...values: unknown[]) {
  for (const value of values) {
    const normalized = toNumber(value)
    if (normalized !== null) return normalized
  }
  return null
}

function formatDate(value: unknown) {
  const text = toText(value)
  if (!text) return null
  const parsed = new Date(text)
  if (Number.isNaN(parsed.getTime())) return text
  return parsed.toLocaleString()
}

function formatHeading(value: unknown) {
  const num = firstNumber(value)
  return num === null ? null : `${Math.round(num)}°`
}

function formatFeet(value: unknown) {
  const num = firstNumber(value)
  return num === null ? null : `${Math.round(num).toLocaleString()} ft`
}

function formatKnots(value: unknown) {
  const num = firstNumber(value)
  return num === null ? null : `${Math.round(num).toLocaleString()} kt`
}

function formatKm(value: unknown) {
  const num = firstNumber(value)
  return num === null ? null : `${num.toFixed(num >= 100 ? 0 : 1)} km`
}

function formatKmPerSecond(value: unknown) {
  const num = firstNumber(value)
  return num === null ? null : `${num.toFixed(2)} km/s`
}

function formatMagnitude(value: unknown) {
  const num = firstNumber(value)
  return num === null ? null : `M ${num.toFixed(1)}`
}

function formatPercent(value: unknown) {
  const num = firstNumber(value)
  return num === null ? null : `${Math.round(num * 100)}%`
}

function formatCoordinates(entity: Record<string, unknown>) {
  const lat = firstNumber(entity.lat)
  const lng = firstNumber(entity.lng)
  if (lat === null || lng === null) return null
  return `${lat.toFixed(4)}, ${lng.toFixed(4)}`
}

function prettifySourceType(sourceType: string) {
  return sourceType
    .replace(/_/g, ' ')
    .replace(/\b\w/g, (char) => char.toUpperCase())
}

function append(fields: DetailRow[], label: string, value: string | null) {
  if (value) fields.push({ label, value })
}

function isFlightEntity(entity: Record<string, unknown>) {
  const sourceType = firstText(entity.sourceType, entity.__kind, entity.type) ?? ''
  return sourceType.includes('flight') || sourceType === 'uavs'
}

function buildGenericDetails(entity: Record<string, unknown>, sourceType: string): DetailPayload {
  const fields: DetailRow[] = []
  append(fields, 'Type', firstText(entity.type, entity.category, entity.source, prettifySourceType(sourceType)))
  append(fields, 'Location', firstText(entity.place, entity.region_name, entity.region, entity.country))
  append(fields, 'Coordinates', formatCoordinates(entity))
  append(fields, 'Time', formatDate(entity.time ?? entity.date ?? entity.published))

  return {
    eyebrow: prettifySourceType(sourceType),
    title: firstText(entity.title, entity.name, entity.callsign, entity.id, entity.mmsi, entity.norad_id) ?? 'Selected object',
    subtitle: firstText(entity.source, entity.datasource, entity.country, entity.company),
    summary: firstText(entity.summary, entity.description, entity.message),
    fields,
    link: firstText(entity.url, entity.source_url, entity.link, entity.media_url),
    linkLabel: firstText(entity.media_url) ? 'Open feed' : 'Open source',
  }
}

export function getSelectedFeatureDetails(entity: unknown): DetailPayload | null {
  if (!isRecord(entity)) return null

  const sourceType = firstText(entity.sourceType, entity.__kind, entity.type) ?? 'object'
  const fields: DetailRow[] = []

  if (isFlightEntity(entity)) {
    append(fields, 'Callsign', firstText(entity.callsign, entity.icao24))
    append(fields, 'Aircraft', firstText(entity.model, entity.icao, entity.registration))
    append(fields, 'Altitude', formatFeet(entity.altitude ?? entity.alt))
    append(fields, 'Speed', formatKnots(entity.speed_knots ?? entity.speed))
    append(fields, 'Route', [firstText(entity.routeOrigin, entity.origin, entity.departure), firstText(entity.routeDestination, entity.destination, entity.arrival)].filter(Boolean).join(' -> ') || null)
    append(fields, 'Airline', getFlightOperator(entity))
    append(fields, 'Heading', formatHeading(entity.heading))
    append(fields, 'Country', firstText(entity.country))

    return {
      eyebrow: sourceType === 'uavs' ? 'UAV' : 'Aircraft',
      title: firstText(entity.callsign, entity.icao24, entity.registration) ?? 'Aircraft',
      subtitle: firstText(getFlightOperator(entity), entity.model, entity.registration),
      summary: firstText(entity.description, entity.wiki),
      fields,
      link: firstText(entity.url, entity.wiki),
      linkLabel: firstText(entity.wiki) ? 'Open reference' : 'Open source',
    }
  }

  if (sourceType === 'ships') {
    append(fields, 'Name', firstText(entity.name, entity.yacht_name))
    append(fields, 'MMSI', firstText(entity.mmsi))
    append(fields, 'Type', firstText(entity.ship_type, entity.type, entity.yacht_category))
    append(fields, 'Speed', formatKnots(entity.speed))
    append(fields, 'Heading', formatHeading(entity.heading))
    append(fields, 'Destination', firstText(entity.destination))
    append(fields, 'Flag', firstText(entity.flag, entity.country))

    return {
      eyebrow: 'Vessel',
      title: firstText(entity.name, entity.yacht_name, entity.mmsi) ?? 'Vessel',
      subtitle: firstText(entity.ship_type, entity.country, entity.yacht_owner),
      summary: firstText(entity.yacht_owner, entity.cargo),
      fields,
      link: firstText(entity.yacht_link),
      linkLabel: 'Open registry',
    }
  }

  if (sourceType === 'earthquakes') {
    append(fields, 'Magnitude', formatMagnitude(entity.magnitude))
    append(fields, 'Depth', formatKm(entity.depth_km))
    append(fields, 'Location', firstText(entity.place))
    append(fields, 'Time', formatDate(entity.time))

    return {
      eyebrow: 'Earthquake',
      title: firstText(entity.place) ?? 'Earthquake',
      subtitle: formatMagnitude(entity.magnitude),
      fields,
      link: firstText(entity.url),
      linkLabel: 'Open bulletin',
    }
  }

  if (sourceType === 'satellites') {
    append(fields, 'Satellite', firstText(entity.name))
    append(fields, 'NORAD ID', firstText(entity.norad_id, entity.id))
    append(fields, 'Altitude', formatKm(entity.altitude_km ?? entity.alt_km))
    append(fields, 'Speed', formatKmPerSecond(entity.speed_km_s))
    append(fields, 'Mission', firstText(entity.mission, entity.sat_type))
    append(fields, 'Country', firstText(entity.country))

    return {
      eyebrow: 'Satellite',
      title: firstText(entity.name) ?? 'Satellite',
      subtitle: firstText(entity.mission, entity.country),
      fields,
    }
  }

  if (sourceType === 'news' || sourceType === 'gdelt' || sourceType === 'liveuamap') {
    append(fields, 'Title', firstText(entity.title))
    append(fields, 'Source', firstText(entity.source, entity.region, entity.datasource, sourceType === 'gdelt' ? 'GDELT' : null))
    append(fields, 'Time', formatDate(entity.published ?? entity.date ?? entity.time))
    append(fields, 'Mentions', firstText(entity.num_mentions))

    return {
      eyebrow: sourceType === 'liveuamap' ? 'Conflict event' : sourceType === 'gdelt' ? 'News event' : 'News',
      title: firstText(entity.title, entity.description) ?? 'Event',
      subtitle: firstText(entity.source, entity.region),
      summary: firstText(entity.summary, entity.description),
      fields,
      link: firstText(entity.url, entity.source_url, entity.link),
      linkLabel: 'Open source',
    }
  }

  if (sourceType === 'firms_fires') {
    append(fields, 'FRP', firstText(entity.frp))
    append(fields, 'Brightness', firstText(entity.brightness))
    append(fields, 'Confidence', firstText(entity.confidence))
    append(fields, 'Time', [firstText(entity.acq_date), firstText(entity.acq_time)].filter(Boolean).join(' ') || null)
    append(fields, 'Satellite', firstText(entity.satellite))
  } else if (sourceType === 'gps_jamming') {
    append(fields, 'Severity', firstText(entity.severity))
    append(fields, 'Degraded share', formatPercent(entity.ratio))
    append(fields, 'Affected tracks', firstText(entity.degraded))
    append(fields, 'Observed tracks', firstText(entity.total))
  } else if (sourceType === 'internet_outages') {
    append(fields, 'Region', firstText(entity.region_name, entity.region))
    append(fields, 'Country', firstText(entity.country_name, entity.country))
    append(fields, 'Severity', firstText(entity.severity))
    append(fields, 'Source', firstText(entity.datasource))
  } else if (sourceType === 'kiwisdr') {
    append(fields, 'Receiver', firstText(entity.name))
    append(fields, 'Bands', firstText(entity.bands))
    append(fields, 'Antenna', firstText(entity.antenna))
    append(fields, 'Users', [firstText(entity.users), firstText(entity.users_max)].filter(Boolean).join(' / ') || null)
  } else if (sourceType === 'datacenters') {
    append(fields, 'Facility', firstText(entity.name))
    append(fields, 'Company', firstText(entity.company))
    append(fields, 'City', firstText(entity.city))
    append(fields, 'Country', firstText(entity.country))
  } else if (sourceType === 'cctv') {
    append(fields, 'Camera', firstText(entity.id))
    append(fields, 'Source', firstText(entity.source))
    append(fields, 'Direction', firstText(entity.direction))
  } else if (sourceType === 'shared_objects' || sourceType === 'simulation_markers' || sourceType === 'fused_objects') {
    append(fields, 'ID', firstText(entity.id))
    append(fields, 'Type', firstText(entity.type, entity.sourceType))
    append(fields, 'Coordinates', formatCoordinates(entity))
  }

  const generic = buildGenericDetails(entity, sourceType)
  return {
    ...generic,
    fields: fields.length > 0 ? [...fields, ...generic.fields.filter((row) => row.label !== 'Type')] : generic.fields,
  }
}

function EmptyState({ compact = false }: { compact?: boolean }) {
  return (
    <div className={compact ? 'text-xs text-slate-400' : 'rounded-2xl border border-white/10 bg-white/5 p-4 text-sm text-slate-400'}>
      Select a marker to inspect its details.
    </div>
  )
}

function DetailsBody({ details, compact = false }: { details: DetailPayload; compact?: boolean }) {
  const rows = compact ? details.fields.slice(0, 5) : details.fields

  return (
    <div className={compact ? 'space-y-2 text-xs' : 'space-y-3'}>
      <div className="space-y-1">
        <div className="text-[10px] font-semibold uppercase tracking-[0.22em] text-cyan-300/90">{details.eyebrow}</div>
        <div className={compact ? 'text-sm font-semibold text-white' : 'text-base font-semibold text-white'}>{details.title}</div>
        {details.subtitle && <div className={compact ? 'text-xs text-slate-400' : 'text-sm text-slate-400'}>{details.subtitle}</div>}
      </div>

      {rows.length > 0 && (
        <div className={`grid gap-2 ${compact ? 'grid-cols-1' : 'grid-cols-1 sm:grid-cols-2'}`}>
          {rows.map((row) => (
            <div key={`${row.label}-${row.value}`} className="rounded-xl border border-white/8 bg-black/20 px-3 py-2">
              <div className="text-[10px] uppercase tracking-[0.18em] text-slate-500">{row.label}</div>
              <div className={compact ? 'mt-1 text-xs text-slate-100' : 'mt-1 text-sm text-slate-100'}>{row.value}</div>
            </div>
          ))}
        </div>
      )}

      {details.summary && <div className={compact ? 'max-w-[260px] text-xs leading-5 text-slate-300' : 'text-sm leading-6 text-slate-300'}>{details.summary}</div>}

      {details.link && (
        <a href={details.link} target="_blank" rel="noreferrer" className="inline-flex rounded-full border border-cyan-400/20 bg-cyan-400/10 px-3 py-1.5 text-[11px] font-medium uppercase tracking-[0.18em] text-cyan-200 transition hover:bg-cyan-400/15">
          {details.linkLabel ?? 'Open link'}
        </a>
      )}
    </div>
  )
}

export function SelectedFeatureDetails({
  entity,
  compact = false,
}: {
  entity: unknown
  compact?: boolean
}) {
  const details = getSelectedFeatureDetails(entity)

  if (compact) {
    if (!details) return <EmptyState compact />
    return <DetailsBody details={details} compact />
  }

  return (
    <PanelCard title="Object Details" subtitle={details?.eyebrow ?? 'Selection-linked details'}>
      {details ? <DetailsBody details={details} /> : <EmptyState />}
    </PanelCard>
  )
}

