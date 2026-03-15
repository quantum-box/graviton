'use client'

import { useState, useEffect } from 'react'
import {
  PanelLeftClose, PanelLeftOpen,
  PanelRightClose, PanelRightOpen,
  Loader2, Satellite, Globe
} from 'lucide-react'

interface ToolbarProps {
  leftOpen: boolean
  rightOpen: boolean
  onToggleLeft: () => void
  onToggleRight: () => void
  isLoading: boolean
}

export function Toolbar({ leftOpen, rightOpen, onToggleLeft, onToggleRight, isLoading }: ToolbarProps) {
  const [time, setTime] = useState('')

  useEffect(() => {
    const update = () => {
      const now = new Date()
      setTime(now.toISOString().replace('T', ' ').slice(0, 19) + ' UTC')
    }
    update()
    const t = setInterval(update, 1000)
    return () => clearInterval(t)
  }, [])

  return (
    <div
      className="flex items-center justify-between px-3 select-none shrink-0"
      style={{
        height: 'var(--toolbar-height)',
        background: 'linear-gradient(180deg, #2d2d4a 0%, #1e1e3a 100%)',
        borderBottom: '1px solid var(--border-color)',
      }}
    >
      {/* Left section: window dots + panel toggle */}
      <div className="flex items-center gap-3">
        <div className="window-controls">
          <span className="window-dot close" />
          <span className="window-dot minimize" />
          <span className="window-dot maximize" />
        </div>
        <button
          onClick={onToggleLeft}
          className="p-1 rounded hover:bg-white/10 transition-colors text-gray-400 hover:text-white"
          title={leftOpen ? 'Hide layers' : 'Show layers'}
        >
          {leftOpen ? <PanelLeftClose size={16} /> : <PanelLeftOpen size={16} />}
        </button>
      </div>

      {/* Center: App title */}
      <div className="flex items-center gap-2">
        <Globe size={16} className="text-cyan-400" />
        <span className="text-sm font-semibold tracking-wider" style={{ color: 'var(--text-accent)' }}>
          GRAVITON
        </span>
        {isLoading && <Loader2 size={14} className="animate-spin text-cyan-400" />}
      </div>

      {/* Right: clock + panel toggle */}
      <div className="flex items-center gap-3">
        <span className="text-xs font-mono" style={{ color: 'var(--text-secondary)' }}>
          {time}
        </span>
        <button
          onClick={onToggleRight}
          className="p-1 rounded hover:bg-white/10 transition-colors text-gray-400 hover:text-white"
          title={rightOpen ? 'Hide intel' : 'Show intel'}
        >
          {rightOpen ? <PanelRightClose size={16} /> : <PanelRightOpen size={16} />}
        </button>
      </div>
    </div>
  )
}
