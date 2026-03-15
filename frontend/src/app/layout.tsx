import type { Metadata, Viewport } from 'next'
import './globals.css'
import { ThemeProvider } from '@/lib/ThemeContext'
import { ErrorBoundary } from '@/components/ErrorBoundary'
import { I18nProvider } from '@/components/I18nProvider'

export const metadata: Metadata = {
  title: 'Graviton — Geospatial Intelligence',
  description: 'Real-time geospatial intelligence dashboard',
}

export const viewport: Viewport = {
  width: 'device-width',
  initialScale: 1,
  maximumScale: 1,
  userScalable: false,
}

export default function RootLayout({
  children,
}: {
  children: React.ReactNode
}) {
  return (
    <html lang="en">
      <body className="antialiased">
        <I18nProvider>
          <ThemeProvider>
            <ErrorBoundary fallbackTitle="Application Shell">{children}</ErrorBoundary>
          </ThemeProvider>
        </I18nProvider>
      </body>
    </html>
  )
}
