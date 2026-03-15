'use client'

import { createContext, useContext, useEffect, useState } from 'react'

type ThemeMode = 'night' | 'day'

const ThemeContext = createContext<{
  theme: ThemeMode
  toggleTheme: () => void
}>({
  theme: 'night',
  toggleTheme: () => {},
})

export function ThemeProvider({ children }: { children: React.ReactNode }) {
  const [theme, setTheme] = useState<ThemeMode>('night')

  useEffect(() => {
    const saved = window.localStorage.getItem('graviton-theme') as ThemeMode | null
    if (saved === 'night' || saved === 'day') {
      setTheme(saved)
      document.documentElement.dataset.theme = saved
    }
  }, [])

  const toggleTheme = () => {
    setTheme((prev) => {
      const next = prev === 'night' ? 'day' : 'night'
      window.localStorage.setItem('graviton-theme', next)
      document.documentElement.dataset.theme = next
      return next
    })
  }

  return <ThemeContext.Provider value={{ theme, toggleTheme }}>{children}</ThemeContext.Provider>
}

export function useTheme() {
  return useContext(ThemeContext)
}

