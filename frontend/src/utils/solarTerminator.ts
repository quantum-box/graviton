export function computeNightPolygon(date: Date = new Date()): GeoJSON.FeatureCollection {
  const deg = Math.PI / 180
  const rad = 180 / Math.PI
  const start = new Date(date.getFullYear(), 0, 0)
  const diff = date.getTime() - start.getTime()
  const day = Math.floor(diff / 86400000)
  const hour = date.getUTCHours() + date.getUTCMinutes() / 60 + date.getUTCSeconds() / 3600
  const gamma = (2 * Math.PI / 365) * (day - 1 + (hour - 12) / 24)

  const eqTime =
    229.18 *
    (0.000075 +
      0.001868 * Math.cos(gamma) -
      0.032077 * Math.sin(gamma) -
      0.014615 * Math.cos(2 * gamma) -
      0.040849 * Math.sin(2 * gamma))

  const declination =
    0.006918 -
    0.399912 * Math.cos(gamma) +
    0.070257 * Math.sin(gamma) -
    0.006758 * Math.cos(2 * gamma) +
    0.000907 * Math.sin(2 * gamma) -
    0.002697 * Math.cos(3 * gamma) +
    0.00148 * Math.sin(3 * gamma)

  const subsolarLng = -(hour - 12) * 15 - eqTime / 4
  const subsolarLat = declination * rad
  const points: [number, number][] = []

  const terminatorLatitude = (lng: number) => {
    const ha = (lng - subsolarLng) * deg
    const tanDec = Math.tan(declination)
    if (Math.abs(tanDec) < 1e-10) return 0
    return Math.atan(-Math.cos(ha) / tanDec) * rad
  }

  for (let lng = -180; lng <= 180; lng += 1) {
    points.push([lng, Math.max(-85, Math.min(85, terminatorLatitude(lng)))])
  }

  const nightIsSouth = subsolarLat > terminatorLatitude(subsolarLng)
  const coords = [...points]
  coords.push([180, nightIsSouth ? -85 : 85], [-180, nightIsSouth ? -85 : 85], coords[0])

  return {
    type: 'FeatureCollection',
    features: [
      {
        type: 'Feature',
        properties: {},
        geometry: { type: 'Polygon', coordinates: [coords] },
      },
    ],
  }
}

