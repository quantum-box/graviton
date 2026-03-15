'use client'

import { useState } from 'react'

export function AuthPanel({
  token,
  user,
  open,
  onClose,
  onAuthenticated,
}: {
  token: string | null
  user: { email?: string; display_name?: string } | null
  open: boolean
  onClose: () => void
  onAuthenticated: (payload: { token: string; user: any }) => void
}) {
  const [mode, setMode] = useState<'login' | 'register'>('login')
  const [email, setEmail] = useState('')
  const [password, setPassword] = useState('')
  const [displayName, setDisplayName] = useState('')
  const [error, setError] = useState<string | null>(null)

  if (!open) return null

  async function submit() {
    setError(null)
    const response = await fetch(`/api/auth/${mode}`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ email, password, display_name: displayName }),
    })
    const data = await response.json()
    if (!response.ok) {
      setError(data.error ?? 'Authentication failed')
      return
    }
    onAuthenticated(data)
    onClose()
  }

  return (
    <div className="fixed inset-0 z-[80] flex items-center justify-center bg-black/60 p-4">
      <div className="w-full max-w-md rounded-[28px] border border-white/10 bg-[#07101a] p-5 shadow-2xl">
        <div className="mb-4 flex items-center justify-between">
          <div>
            <div className="text-[11px] uppercase tracking-[0.22em] text-slate-400">Operator Access</div>
            <div className="mt-1 text-lg text-white">{token ? user?.display_name ?? user?.email : mode === 'login' ? 'Login' : 'Create account'}</div>
          </div>
          <button onClick={onClose} className="rounded-full border border-white/10 px-3 py-1 text-xs text-slate-300">Close</button>
        </div>
        {!token && (
          <div className="mb-4 flex gap-2">
            <button onClick={() => setMode('login')} className={`rounded-full px-3 py-1 text-xs ${mode === 'login' ? 'bg-teal-400/20 text-teal-100' : 'bg-white/5 text-slate-400'}`}>Login</button>
            <button onClick={() => setMode('register')} className={`rounded-full px-3 py-1 text-xs ${mode === 'register' ? 'bg-teal-400/20 text-teal-100' : 'bg-white/5 text-slate-400'}`}>Register</button>
          </div>
        )}
        {!token ? (
          <div className="space-y-3">
            {mode === 'register' && (
              <input value={displayName} onChange={(e) => setDisplayName(e.target.value)} placeholder="Display name" className="w-full rounded-2xl border border-white/10 bg-white/5 px-3 py-2 text-sm text-white outline-none" />
            )}
            <input value={email} onChange={(e) => setEmail(e.target.value)} placeholder="Email" className="w-full rounded-2xl border border-white/10 bg-white/5 px-3 py-2 text-sm text-white outline-none" />
            <input value={password} onChange={(e) => setPassword(e.target.value)} type="password" placeholder="Password" className="w-full rounded-2xl border border-white/10 bg-white/5 px-3 py-2 text-sm text-white outline-none" />
            {error && <div className="text-sm text-rose-300">{error}</div>}
            <button onClick={() => void submit()} className="w-full rounded-2xl bg-teal-400/20 px-4 py-2 text-sm text-teal-100">
              {mode === 'login' ? 'Login' : 'Create account'}
            </button>
          </div>
        ) : (
          <div className="space-y-2 text-sm text-slate-300">
            <div>{user?.email}</div>
            <div>JWT session is stored locally for team sharing and C2 actions.</div>
          </div>
        )}
      </div>
    </div>
  )
}
