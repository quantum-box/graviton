'use client'

import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'

interface FeedConfig {
  name: string
  url: string
  weight: number
}

export function SettingsPanel({ open, onClose, token }: { open: boolean; onClose: () => void; token: string | null }) {
  const [adminKey, setAdminKey] = useState('')
  const [feeds, setFeeds] = useState<FeedConfig[]>([])
  const [apiKeys, setApiKeys] = useState<Record<string, string>>({})
  const [webhookUrl, setWebhookUrl] = useState('')
  const { t } = useTranslation()

  useEffect(() => {
    if (!open) return
    fetch('/api/settings/news-feeds')
      .then((resp) => resp.json())
      .then((data) => setFeeds(data.feeds ?? []))
      .catch(() => setFeeds([]))
    if (token) {
      fetch('/api/team/webhook', { headers: { Authorization: `Bearer ${token}` } })
        .then((resp) => (resp.ok ? resp.json() : { webhook_url: '' }))
        .then((data) => setWebhookUrl(data.webhook_url ?? ''))
        .catch(() => setWebhookUrl(''))
    }
  }, [open, token])

  if (!open) return null

  async function saveFeeds() {
    await fetch('/api/settings/news-feeds', {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json', 'x-admin-key': adminKey },
      body: JSON.stringify({ feeds }),
    })
  }

  async function saveKeys() {
    await fetch('/api/settings/api-keys', {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json', 'x-admin-key': adminKey },
      body: JSON.stringify(apiKeys),
    })
  }

  async function saveWebhook() {
    if (!token) return
    await fetch('/api/team/webhook', {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json', Authorization: `Bearer ${token}` },
      body: JSON.stringify({ webhook_url: webhookUrl }),
    })
  }

  return (
    <div className="fixed inset-0 z-50 flex justify-end bg-black/60">
      <div className="flex h-full w-full max-w-xl flex-col border-l border-white/10 bg-[#07101a] p-4 shadow-2xl">
        <div className="mb-4 flex items-center justify-between">
          <div className="text-sm font-semibold uppercase tracking-[0.2em] text-cyan-300">{t('settings.title')}</div>
          <button onClick={onClose} className="rounded-full border border-white/10 px-3 py-1 text-xs text-slate-300">
            {t('settings.close')}
          </button>
        </div>
        <input
          value={adminKey}
          onChange={(e) => setAdminKey(e.target.value)}
          placeholder={t('settings.adminPlaceholder')}
          className="mb-4 rounded-2xl border border-white/10 bg-white/5 px-3 py-2 text-sm outline-none"
        />
        <div className="mb-4">
          <div className="mb-2 text-xs uppercase tracking-[0.18em] text-slate-400">{t('settings.rssFeeds')}</div>
          <div className="space-y-2">
            {feeds.map((feed, index) => (
              <div key={`${feed.name}-${index}`} className="grid grid-cols-1 gap-2 rounded-2xl border border-white/10 bg-white/5 p-3">
                <input value={feed.name} onChange={(e) => setFeeds(feeds.map((item, i) => (i === index ? { ...item, name: e.target.value } : item)))} className="bg-transparent text-sm outline-none" />
                <input value={feed.url} onChange={(e) => setFeeds(feeds.map((item, i) => (i === index ? { ...item, url: e.target.value } : item)))} className="bg-transparent text-xs outline-none text-slate-400" />
              </div>
            ))}
          </div>
          <button onClick={saveFeeds} className="mt-3 rounded-2xl bg-cyan-500/20 px-4 py-2 text-sm text-cyan-200">
            {t('settings.saveFeeds')}
          </button>
        </div>
        <div>
          <div className="mb-2 text-xs uppercase tracking-[0.18em] text-slate-400">{t('settings.apiKeys')}</div>
          <div className="space-y-2">
            {['AIS_API_KEY', 'OPENSKY_CLIENT_ID', 'OPENSKY_CLIENT_SECRET'].map((key) => (
              <input
                key={key}
                placeholder={key}
                value={apiKeys[key] ?? ''}
                onChange={(e) => setApiKeys((prev) => ({ ...prev, [key]: e.target.value }))}
                className="w-full rounded-2xl border border-white/10 bg-white/5 px-3 py-2 text-sm outline-none"
              />
            ))}
          </div>
          <button onClick={saveKeys} className="mt-3 rounded-2xl bg-cyan-500/20 px-4 py-2 text-sm text-cyan-200">
            {t('settings.saveKeys')}
          </button>
        </div>
        <div className="mt-6">
          <div className="mb-2 text-xs uppercase tracking-[0.18em] text-slate-400">Discord Webhook</div>
          <input
            placeholder="https://discord.com/api/webhooks/..."
            value={webhookUrl}
            onChange={(e) => setWebhookUrl(e.target.value)}
            className="w-full rounded-2xl border border-white/10 bg-white/5 px-3 py-2 text-sm outline-none"
          />
          <button onClick={saveWebhook} disabled={!token} className="mt-3 rounded-2xl bg-emerald-500/20 px-4 py-2 text-sm text-emerald-200 disabled:opacity-50">
            Save webhook
          </button>
        </div>
        <a href="/api/docs" target="_blank" className="mt-6 inline-flex rounded-2xl border border-white/10 px-4 py-2 text-sm text-slate-300">
          Open API docs
        </a>
      </div>
    </div>
  )
}
