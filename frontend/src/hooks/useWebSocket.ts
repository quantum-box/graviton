'use client'

import { useEffect, useRef, useState } from 'react'

export interface LiveSocketState {
  connected: boolean
  latestEvent: any
  alerts: any[]
  datasets: Record<string, any>
}

const EMPTY_STATE: LiveSocketState = {
  connected: false,
  latestEvent: null,
  alerts: [],
  datasets: {},
}

export function useWebSocket() {
  const [state, setState] = useState<LiveSocketState>(EMPTY_STATE)
  const reconnectRef = useRef<number | null>(null)

  useEffect(() => {
    let active = true
    let socket: WebSocket | null = null

    const connect = () => {
      const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:'
      socket = new WebSocket(`${protocol}//${window.location.hostname}:8000/api/ws/live`)

      socket.onopen = () => {
        if (!active) return
        setState((prev) => ({ ...prev, connected: true }))
      }

      socket.onmessage = (event) => {
        if (!active) return
        try {
          const message = JSON.parse(event.data)
          setState((prev) => {
            if (message.type === 'snapshot') {
              return {
                connected: true,
                latestEvent: message,
                alerts: message.fast?.alerts ?? prev.alerts,
                datasets: prev.datasets,
              }
            }
            if (message.type === 'alert') {
              return {
                ...prev,
                latestEvent: message,
                alerts: [message.alert, ...prev.alerts].slice(0, 20),
              }
            }
            if (message.type === 'dataset') {
              return {
                ...prev,
                latestEvent: message,
                datasets: { ...prev.datasets, [message.key]: message.payload },
              }
            }
            if (message.type === 'simulation') {
              return {
                ...prev,
                latestEvent: message,
                datasets: { ...prev.datasets, simulation_markers: message.payload },
              }
            }
            return { ...prev, latestEvent: message }
          })
        } catch (error) {
          console.warn(error)
        }
      }

      socket.onclose = () => {
        if (!active) return
        setState((prev) => ({ ...prev, connected: false }))
        reconnectRef.current = window.setTimeout(connect, 1500)
      }
    }

    connect()
    return () => {
      active = false
      if (reconnectRef.current) window.clearTimeout(reconnectRef.current)
      socket?.close()
    }
  }, [])

  return state
}
