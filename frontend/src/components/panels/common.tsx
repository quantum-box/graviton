import { useTranslation } from 'react-i18next'

export function PanelCard({ title, subtitle, children }: { title: string; subtitle?: string; children: React.ReactNode }) {
  return (
    <section className="rounded-xl border border-white/10 bg-white/[0.03] p-3">
      <div className="mb-2">
        <div className="text-[11px] uppercase tracking-[0.2em] text-cyan-300">{title}</div>
        {subtitle && <div className="text-xs text-slate-500">{subtitle}</div>}
      </div>
      <div className="space-y-2 text-sm">{children}</div>
    </section>
  )
}

export function CompactList({ items }: { items: Array<{ title: string; meta?: string; href?: string }> }) {
  const { t } = useTranslation()

  if (items.length === 0) {
    return <div className="rounded-lg bg-black/20 p-2 text-sm text-slate-500">{t('panels.noItems')}</div>
  }

  return (
    <div className="space-y-2">
      {items.slice(0, 8).map((item, idx) => (
        <div key={`${item.title}-${idx}`} className="rounded-lg bg-black/20 p-2">
          {item.href ? (
            <a href={item.href} target="_blank" className="block text-slate-100 hover:text-cyan-300">
              {item.title}
            </a>
          ) : (
            <div className="text-slate-100">{item.title}</div>
          )}
          {item.meta && <div className="text-xs text-slate-500">{item.meta}</div>}
        </div>
      ))}
    </div>
  )
}
