import type { Metadata } from 'next'
import './globals.css'
import { ThemeProvider } from '@/lib/ThemeContext'
import { ErrorBoundary } from '@/components/ErrorBoundary'

export const metadata: Metadata = {
  title: 'Graviton — Geospatial Intelligence',
  description: 'Real-time geospatial intelligence dashboard',
}

export default function RootLayout({
  children,
}: {
  children: React.ReactNode
}) {
  return (
    <html lang="en">
      <body className="antialiased">
        <ThemeProvider>
          <ErrorBoundary fallbackTitle="Application Shell">{children}</ErrorBoundary>
        </ThemeProvider>
      </body>
    </html>
  )
}
