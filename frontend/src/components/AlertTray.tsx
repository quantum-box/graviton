'use client'

export function AlertTray({ alerts }: { alerts: any[] }) {
  if (alerts.length === 0) return null
  return (
    <div className="pointer-events-auto absolute right-3 top-16 z-30 hidden w-[320px] space-y-2 lg:block">
      {alerts.slice(0, 4).map((alert, index) => (
        <div key={`${alert.id ?? index}`} className="rounded-2xl border border-amber-300/20 bg-[rgba(31,18,10,0.86)] p-3 shadow-lg backdrop-blur-xl">
          <div className="flex items-center justify-between gap-2">
            <div className="text-[11px] uppercase tracking-[0.18em] text-amber-200">{alert.severity ?? 'info'}</div>
            <div className="text-[10px] text-slate-400">{alert.source}</div>
          </div>
          <div className="mt-1 text-sm text-white">{alert.title}</div>
          <div className="mt-1 text-xs text-slate-300">{alert.message}</div>
        </div>
      ))}
    </div>
  )
}
