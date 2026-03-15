const AIRLINE_CODES: Record<string, string> = {
  AAL: 'American Airlines',
  ACA: 'Air Canada',
  AFR: 'Air France',
  AIC: 'Air India',
  ANA: 'All Nippon Airways',
  BAW: 'British Airways',
  CCA: 'Air China',
  CES: 'China Eastern Airlines',
  CPA: 'Cathay Pacific',
  DAL: 'Delta Air Lines',
  DLH: 'Lufthansa',
  EIN: 'Aer Lingus',
  ETD: 'Etihad Airways',
  ETH: 'Ethiopian Airlines',
  EVA: 'EVA Air',
  ICE: 'Icelandair',
  JAL: 'Japan Airlines',
  JBU: 'JetBlue',
  KAL: 'Korean Air',
  KLM: 'KLM',
  QFA: 'Qantas',
  QTR: 'Qatar Airways',
  RJA: 'Royal Jordanian',
  RYR: 'Ryanair',
  SAS: 'Scandinavian Airlines',
  SIA: 'Singapore Airlines',
  SWR: 'Swiss',
  THY: 'Turkish Airlines',
  UAE: 'Emirates',
  UAL: 'United Airlines',
  UPS: 'UPS Airlines',
  VIR: 'Virgin Atlantic',
  WJA: 'WestJet',
}

export function getAirlineName(value?: string | null) {
  if (!value) return null
  const normalized = value.trim().toUpperCase()
  const prefix = normalized.match(/^[A-Z]{3}/)?.[0]
  if (!prefix) return null
  return AIRLINE_CODES[prefix] ?? null
}

export function getFlightOperator(entity: Record<string, unknown> | null | undefined) {
  if (!entity) return null
  const explicitOperator = entity.operator
  if (typeof explicitOperator === 'string' && explicitOperator.trim()) {
    return explicitOperator
  }

  const airlineFromCallsign = getAirlineName(typeof entity.callsign === 'string' ? entity.callsign : null)
  if (airlineFromCallsign) return airlineFromCallsign

  const airlineFromTrackedName = getAirlineName(typeof entity.tracked_name === 'string' ? entity.tracked_name : null)
  if (airlineFromTrackedName) return airlineFromTrackedName

  return null
}
