export function SatelliteIcon({ color = '#fde047' }: { color?: string }) {
  return (
    <svg width="18" height="18" viewBox="0 0 24 24" fill="none">
      <path d="M9 3 6 6l2 2-3 3 2 2 3-3 2 2 3-3-7-7Z" fill={color} />
      <path d="M14 10 21 3l-1-1-7 7M10 14 3 21l1 1 7-7" stroke={color} strokeWidth="1.5" />
    </svg>
  )
}

