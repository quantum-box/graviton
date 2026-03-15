'use client'

interface AdvancedFilterModalProps {
  open: boolean
  title: string
  options: string[]
  selected: string[]
  onClose: () => void
  onChange: (next: string[]) => void
}

export function AdvancedFilterModal({ open, title, options, selected, onClose, onChange }: AdvancedFilterModalProps) {
  if (!open) return null
  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/70 px-4">
      <div className="w-full max-w-lg rounded-3xl border border-white/10 bg-[#08111c] p-4 shadow-2xl">
        <div className="mb-3 flex items-center justify-between">
          <div className="text-sm font-semibold tracking-[0.18em] text-cyan-300 uppercase">{title}</div>
          <button onClick={onClose} className="rounded-full border border-white/10 px-3 py-1 text-xs text-slate-300">
            Close
          </button>
        </div>
        <div className="grid max-h-[50vh] grid-cols-1 gap-2 overflow-y-auto sm:grid-cols-2">
          {options.map((option) => {
            const enabled = selected.includes(option)
            return (
              <button
                key={option}
                onClick={() => onChange(enabled ? selected.filter((item) => item !== option) : [...selected, option])}
                className="rounded-2xl border px-3 py-2 text-left text-sm"
                style={{
                  borderColor: enabled ? '#22d3ee' : 'rgba(255,255,255,0.08)',
                  background: enabled ? 'rgba(34,211,238,0.12)' : 'rgba(255,255,255,0.03)',
                }}
              >
                {option}
              </button>
            )
          })}
        </div>
      </div>
    </div>
  )
}

