'use client'

import { useTranslation } from 'react-i18next'

export default function GlobalError({
  error,
  reset,
}: {
  error: Error & { digest?: string }
  reset: () => void
}) {
  console.error(error)
  const { t } = useTranslation()

  return (
    <html lang="en">
      <body className="flex min-h-screen items-center justify-center bg-slate-950 p-6 text-slate-100">
        <div className="max-w-lg rounded-[28px] border border-red-500/20 bg-red-950/20 p-6">
          <div className="text-[11px] uppercase tracking-[0.28em] text-red-300">{t('globalError.title')}</div>
          <h1 className="mt-3 text-2xl font-semibold">{t('globalError.heading')}</h1>
          <p className="mt-3 text-sm text-slate-300">
            {t('globalError.body')}
          </p>
          <button onClick={reset} className="mt-5 rounded-2xl bg-red-400 px-4 py-2 text-sm font-medium text-slate-950">
            {t('globalError.retry')}
          </button>
        </div>
      </body>
    </html>
  )
}
