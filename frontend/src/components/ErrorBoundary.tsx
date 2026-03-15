'use client'

import { AlertTriangle } from 'lucide-react'
import { Component, type ErrorInfo, type ReactNode } from 'react'
import i18n from '@/lib/i18n'

interface ErrorBoundaryProps {
  children: ReactNode
  fallbackTitle?: string
}

interface ErrorBoundaryState {
  hasError: boolean
}

export class ErrorBoundary extends Component<ErrorBoundaryProps, ErrorBoundaryState> {
  state: ErrorBoundaryState = { hasError: false }

  static getDerivedStateFromError() {
    return { hasError: true }
  }

  componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    console.error('Graviton UI error', error, errorInfo)
  }

  render() {
    if (!this.state.hasError) {
      return this.props.children
    }

    return (
      <div className="flex min-h-[280px] items-center justify-center p-4">
        <div className="max-w-md rounded-3xl border border-amber-500/30 bg-[#1a140a]/90 p-5 text-sm text-amber-50 shadow-2xl">
          <div className="mb-2 flex items-center gap-2 text-[11px] uppercase tracking-[0.24em] text-amber-300">
            <AlertTriangle size={14} />
            {this.props.fallbackTitle ?? i18n.t('errorBoundary.defaultTitle')}
          </div>
          <div className="text-base font-semibold">{i18n.t('errorBoundary.panelCrashed')}</div>
          <p className="mt-2 text-amber-100/80">
            {i18n.t('errorBoundary.reload')}
          </p>
        </div>
      </div>
    )
  }
}
