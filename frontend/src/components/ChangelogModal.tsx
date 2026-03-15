'use client'

import { History, X } from 'lucide-react'

interface ChangelogModalProps {
  open: boolean
  onClose: () => void
}

const ITEMS = [
  'Responsive drawers for sources, intel panels, and mobile utility controls.',
  'First-run onboarding and in-app changelog access from the toolbar.',
  'Map scale bar, airline code expansion, and isolated React error boundary fallback.',
  'Docker, startup script, and GitHub Actions paths aligned with backend and frontend build checks.',
]

export function ChangelogModal({ open, onClose }: ChangelogModalProps) {
  if (!open) return null

  return (
    <div className="fixed inset-0 z-[70] flex items-center justify-center bg-slate-950/70 p-4 backdrop-blur-sm">
      <div className="w-full max-w-2xl rounded-[28px] border border-white/10 bg-[#08111b] shadow-[0_20px_70px_rgba(0,0,0,0.5)]">
        <div className="flex items-start justify-between gap-3 border-b border-white/10 px-6 py-5">
          <div>
            <div className="flex items-center gap-2 text-[11px] uppercase tracking-[0.28em] text-cyan-300">
              <History size={14} />
              Changelog
            </div>
            <h2 className="mt-2 text-xl font-semibold text-slate-100">Current delivery</h2>
          </div>
          <button onClick={onClose} className="rounded-full border border-white/10 p-2 text-slate-300 hover:bg-white/5" aria-label="Close changelog">
            <X size={16} />
          </button>
        </div>
        <div className="space-y-3 px-6 py-5">
          {ITEMS.map((item) => (
            <div key={item} className="rounded-2xl border border-white/10 bg-white/[0.03] px-4 py-3 text-sm text-slate-300">
              {item}
            </div>
          ))}
        </div>
      </div>
    </div>
  )
}

