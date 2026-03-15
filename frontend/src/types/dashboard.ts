export interface BaseEntity {
  id?: string | number
  lat?: number
  lng?: number
  type?: string
  title?: string
  name?: string
  [key: string]: unknown
}

export interface FastData {
  last_updated?: string
  commercial_flights: BaseEntity[]
  private_flights: BaseEntity[]
  private_jets: BaseEntity[]
  military_flights: BaseEntity[]
  tracked_flights: BaseEntity[]
  uavs: BaseEntity[]
  gps_jamming: BaseEntity[]
  ships: BaseEntity[]
  satellites: BaseEntity[]
}

export interface SlowData {
  last_updated?: string
  earthquakes: BaseEntity[]
  news: BaseEntity[]
  stocks: BaseEntity[]
  oil: BaseEntity[]
  firms_fires: BaseEntity[]
  gdelt: BaseEntity[]
  liveuamap: BaseEntity[]
  frontlines: GeoJSON.FeatureCollection
  space_weather?: Record<string, unknown>
  weather?: Record<string, unknown>
  internet_outages: BaseEntity[]
  kiwisdr: BaseEntity[]
  datacenters: BaseEntity[]
  cctv: BaseEntity[]
}

export interface DashboardData {
  fastData: FastData | null
  slowData: SlowData | null
}

export interface FocusLocation {
  lat: number
  lng: number
  label?: string
}

