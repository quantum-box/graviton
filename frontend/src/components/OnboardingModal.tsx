'use client'

import { Compass, Filter, Layers3, Shield, X } from 'lucide-react'
import { useTranslation } from 'react-i18next'

interface OnboardingModalProps {
  open: boolean
  onClose: () => void
}

export function OnboardingModal({ open, onClose }: OnboardingModalProps) {
  const { t } = useTranslation()
  if (!open) return null

  const steps = [
    { icon: Layers3, title: t('onboarding.step1Title'), body: t('onboarding.step1Body') },
    { icon: Compass, title: t('onboarding.step2Title'), body: t('onboarding.step2Body') },
    { icon: Filter, title: t('onboarding.step3Title'), body: t('onboarding.step3Body') },
    { icon: Shield, title: t('onboarding.step4Title'), body: t('onboarding.step4Body') },
  ]

  return (
    <div className="fixed inset-0 z-[70] flex items-center justify-center bg-slate-950/70 p-4 backdrop-blur-sm">
      <div className="w-full max-w-3xl overflow-hidden rounded-[28px] border border-cyan-400/20 bg-[linear-gradient(135deg,#06101a,#0b1c2b_55%,#102a3d)] shadow-[0_24px_80px_rgba(0,0,0,0.45)]">
        <div className="flex items-start justify-between gap-4 border-b border-white/10 px-6 py-5">
          <div>
            <div className="text-[11px] uppercase tracking-[0.28em] text-cyan-300">{t('onboarding.title')}</div>
            <h2 className="mt-2 text-2xl font-semibold text-slate-50">{t('onboarding.heading')}</h2>
            <p className="mt-2 max-w-xl text-sm text-slate-300">
              {t('onboarding.intro')}
            </p>
          </div>
          <button onClick={onClose} className="rounded-full border border-white/10 p-2 text-slate-300 hover:bg-white/5" aria-label={t('onboarding.closeAria')}>
            <X size={16} />
          </button>
        </div>

        <div className="grid gap-4 px-6 py-6 md:grid-cols-2">
          {steps.map(({ icon: Icon, title, body }) => (
            <div key={title} className="rounded-3xl border border-white/10 bg-white/[0.04] p-4">
              <div className="mb-3 inline-flex rounded-2xl border border-cyan-400/20 bg-cyan-400/10 p-2 text-cyan-300">
                <Icon size={18} />
              </div>
              <div className="text-base font-semibold text-slate-100">{title}</div>
              <p className="mt-2 text-sm leading-6 text-slate-400">{body}</p>
            </div>
          ))}
        </div>

        <div className="flex items-center justify-between gap-3 border-t border-white/10 px-6 py-4">
          <p className="text-xs text-slate-500">{t('onboarding.footer')}</p>
          <button onClick={onClose} className="rounded-2xl bg-cyan-400 px-4 py-2 text-sm font-medium text-slate-950">
            {t('onboarding.enter')}
          </button>
        </div>
      </div>
    </div>
  )
}
