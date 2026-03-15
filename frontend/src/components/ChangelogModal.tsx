'use client'

import { History, X } from 'lucide-react'
import { useTranslation } from 'react-i18next'

interface ChangelogModalProps {
  open: boolean
  onClose: () => void
}

export function ChangelogModal({ open, onClose }: ChangelogModalProps) {
  const { t } = useTranslation()
  if (!open) return null

  const items = [t('changelog.item1'), t('changelog.item2'), t('changelog.item3'), t('changelog.item4')]

  return (
    <div className="fixed inset-0 z-[70] flex items-center justify-center bg-slate-950/70 p-4 backdrop-blur-sm">
      <div className="w-full max-w-2xl rounded-[28px] border border-white/10 bg-[#08111b] shadow-[0_20px_70px_rgba(0,0,0,0.5)]">
        <div className="flex items-start justify-between gap-3 border-b border-white/10 px-6 py-5">
          <div>
            <div className="flex items-center gap-2 text-[11px] uppercase tracking-[0.28em] text-cyan-300">
              <History size={14} />
              {t('changelog.title')}
            </div>
            <h2 className="mt-2 text-xl font-semibold text-slate-100">{t('changelog.heading')}</h2>
          </div>
          <button onClick={onClose} className="rounded-full border border-white/10 p-2 text-slate-300 hover:bg-white/5" aria-label={t('changelog.closeAria')}>
            <X size={16} />
          </button>
        </div>
        <div className="space-y-3 px-6 py-5">
          {items.map((item) => (
            <div key={item} className="rounded-2xl border border-white/10 bg-white/[0.03] px-4 py-3 text-sm text-slate-300">
              {item}
            </div>
          ))}
        </div>
      </div>
    </div>
  )
}
