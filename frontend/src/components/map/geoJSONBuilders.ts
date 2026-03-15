import type { BaseEntity } from '@/types/dashboard'

export function pointCollection(items: BaseEntity[], kind: string): GeoJSON.FeatureCollection {
  return {
    type: 'FeatureCollection',
    features: items
      .filter((item) => typeof item.lng === 'number' && typeof item.lat === 'number')
      .map((item, index) => ({
        type: 'Feature',
        properties: { ...item, __kind: kind, __id: item.id ?? `${kind}-${index}` },
        geometry: { type: 'Point', coordinates: [item.lng as number, item.lat as number] },
      })),
  }
}

