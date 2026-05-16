import { useCallback, useEffect, useRef, useState } from "react"

export interface DashboardSnapshot {
    total_player: number
    connected_players: number
    lobby_number: number
    game_number: number
}

// interface MatchSummary {
//     match_id: string
//     winner: string
//     mmr_delta: number
// }

export type ConnectionStatus = "disconnected" | "connecting" | "connected" | "error"

interface UseDashboardOptions {
    port?: number
    host?: string
}

export function useDashboard({ port = 12345, host = "127.0.0.1" }: UseDashboardOptions = {}) {
    const [data, setData] = useState<DashboardSnapshot | null>(null)
    const [status, setStatus] = useState<ConnectionStatus>("disconnected")
    const wsRef = useRef<WebSocket | null>(null)
    const shouldReconnect = useRef(true)

    const connect = useCallback(() => {
        if (wsRef.current) {
            wsRef.current.close()
        }

        setStatus("connecting")
        shouldReconnect.current = true

        const ws = new WebSocket(`ws://${host}:${port}/ws/dashboard`)
        wsRef.current = ws

        ws.onopen = () => {
            setStatus("connected")
        }

        ws.onclose = () => {
            setStatus("disconnected")
            wsRef.current = null
        }

        ws.onerror = () => {
            setStatus("error")
        }

        ws.onmessage = (event) => {
            try {
                const snapshot = JSON.parse(event.data) as DashboardSnapshot
                setData(snapshot)
            } catch {
                console.error("Failed to parse snapshot", event.data)
            }
        }
    }, [host, port])

    const disconnect = useCallback(() => {
        shouldReconnect.current = false
        wsRef.current?.close()
        setStatus("disconnected")
        setData(null)
    }, [])

    useEffect(() => {
        connect()
        return () => {
            shouldReconnect.current = false
            wsRef.current?.close()
        }
    }, [connect])

    return { data, status, connect, disconnect }
}
