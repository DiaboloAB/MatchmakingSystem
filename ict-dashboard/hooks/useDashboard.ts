import { useCallback, useEffect, useRef, useState } from "react"

//{"type":"Snapshot","snapshot":{"total_player":96,"total_finished_game":89,"connected_players":13,"lobby_number":12,"game_number":1,"queueing_lobbies":2,"waiting_games":3,"ongoing_games":1}}
export interface DashboardSnapshot {
    total_player: number
    total_finished_game: number
    connected_players: number
    lobby_number: number
    game_number: number
    queueing_lobbies: number
    waiting_games: number
    ongoing_games: number
    average_queue_time: number
    successful_match_rate: number
}

//{ "type": "PlayerList", "players": [{"id":"dd693c40-f2b7-48b6-b1d3-cc232cb5a017","name":"ultra-brothers","mmr":1000.0,"true_skill":1000.0,"status":{"NeedConfirmation":{"game_id":"e9e3b91a-fc12-4174-a5b7-ad9dbf3ddd14"}},"lobby":"bcdadfbf-217c-4f1f-9152-d5097bba360a","wins":[],"losses":[],"skills":["lewd","steepness","boletes","lateness"]}]} }
export interface PlayerInfo {
    id: string
    name: string
    mmr: number
    true_skill: number
    wins: string[]
    losses: string[]
    // status: "Idle" | "InQueue" | "NeedConfirmation" | "InGame"
    status: "Idle" | "InQueue" | { NeedConfirmation: { game_id: string } } | { InGame: { game_id: string } }
    lobby: string | null

    debug_rank: string
    debug_player_level: number
    debug_player_form: number
}

// {"type":"LobbyList","lobbys":[{"id":"91bcddaf-0245-43de-8ef4-c9e8f20f2b92","players":["b3c4d938-9b8d-46b3-9568-8b1f4b3596f8"],"owner":"b3c4d938-9b8d-46b3-9568-8b1f4b3596f8","status":{"InQueue":{}}}],"queueing_lobby":[{"lobby_id":"91bcddaf-0245-43de-8ef4-c9e8f20f2b92","avg_mmr":1000.0,"player_count":1,"queue_seconds":3}]}
export interface LobbyInfo {
    id: string
    players: string[]
    owner: string
    status: "InQueue" | "NeedConfirmation" | "InGame"
}

export interface QueueInfo {
    lobby_id: string
    avg_mmr: number
    player_count: number
    queue_seconds: number
}

//{"type":"GameList","waiting_games":[{"id":"495752b9-e693-4cd3-8527-5be4bcee3902","team1":["d43f53f4-e134-4169-9701-28fb1389b7d8"],"team2":["f9f7c5c9-c4ce-4d29-af53-fdbb60d478d2"],"lobbys":["b6b6c189-c12b-4c6e-ac84-603f2d9c82bf","91b99263-69e7-432d-a58f-3bfa090b2d08"],"confirmed":["f9f7c5c9-c4ce-4d29-af53-fdbb60d478d2"],"elapsed_seconds":211}],"ongoing_games":[]}

export interface GameInfo {
    id: string
    team1: string[]
    team2: string[]
    lobbys: string[]
    confirmed: string[]
    elapsed_seconds: number
}

//{"type":"SettingsUpdate","settings":{"lobby_capacity":1,"team_size":1,"simulation_speed":20.0,"confirmation_time":-1.0}}

export interface AppSettings {
    lobby_capacity: number
    team_size: number
    simulation_speed: number
    confirmation_time: number
    matchmaking_delta: number
    matchmaking_time_factor: number
}


//{"type":"NewGameResult","game":{"id":"77c91534-07cb-48f6-82e1-5e8bad1e00cf","team1":["9847ea1e-9eaf-4531-8f19-dc383cc4ef24"],"team2":["f668aff0-1999-442f-8450-68408946319e"],"winner":2}}
export interface GameResult {
    id: string
    team1: string[]
    team2: string[]
    winner: number

    // dashboard-only fields
    time: string
}


export type ConnectionStatus = "disconnected" | "connecting" | "connected" | "error"

interface UseDashboardOptions {
    port?: number
    host?: string
}

export function useDashboard({ port = 12345, host = "127.0.0.1" }: UseDashboardOptions = {}) {
    const [data, setData] = useState<DashboardSnapshot | null>(null)
    const [playerList, setPlayerList] = useState<PlayerInfo[]>([])
    const [lobbyList, setLobbyList] = useState<LobbyInfo[]>([])
    const [queueList, setQueueList] = useState<QueueInfo[]>([])
    const [waitingGames, setWaitingGames] = useState<GameInfo[]>([])
    const [gameList, setGameList] = useState<GameInfo[]>([])
    const [gameResults, setGameResults] = useState<GameResult[]>([])
    const [status, setStatus] = useState<ConnectionStatus>("disconnected")
    const [settings, setSettings] = useState<AppSettings | null>(null)
    const wsRef = useRef<WebSocket | null>(null)
    const shouldReconnect = useRef(true)

    const connect = useCallback(() => {
        if (wsRef.current) {
            // 1. Strip the old listener before closing so it doesn't trigger state updates
            wsRef.current.onclose = null
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
            if (wsRef.current === ws) {
                setStatus("disconnected")
                wsRef.current = null
            }
        }

        ws.onerror = () => {
            if (wsRef.current === ws) {
                setStatus("error")
            }
        }

        ws.onmessage = (event) => {
            console.log("Received message:", event.data)
            try {
                const message = JSON.parse(event.data)

                if (message.type === "Snapshot") {
                    setData(message.snapshot)
                }
                if (message.type === "PlayerList") {
                    setPlayerList(message.players)
                }
                if (message.type === "LobbyList") {
                    setLobbyList(message.lobbys)
                    setQueueList(message.queueing_lobby)
                }
                if (message.type === "GameList") {
                    setWaitingGames(message.waiting_games)
                    setGameList(message.ongoing_games)
                }
                if (message.type === "SettingsUpdate") {
                    setSettings(message.settings)
                }
                if (message.type === "NewGameResult") {
                    message.game.time = new Date().toLocaleTimeString()
                    setGameResults(prev => [...prev, message.game])
                }
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
        setSettings(null)
    }, [])

    useEffect(() => {
        connect()
        return () => {
            shouldReconnect.current = false
            wsRef.current?.close()
        }
    }, [connect])

    // reconnect when pressing Ctrl+ Shift+R
    useEffect(() => {
        const handleKeyDown = (event: KeyboardEvent) => {
            if (event.ctrlKey && event.shiftKey && event.key === "R") {
                connect()
            }
        }
        window.addEventListener("keydown", handleKeyDown)
        return () => {
            window.removeEventListener("keydown", handleKeyDown)
        }
    }, [connect])

    const updateSettings = useCallback((newSettings: AppSettings) => {
        if (wsRef.current && wsRef.current.readyState === WebSocket.OPEN) {
            wsRef.current.send(JSON.stringify({
                type: "UpdateSettings",
                settings: newSettings
            }))
        } else {
            console.error("WebSocket is not connected")
        }
    }, [])

    return { data, playerList, lobbyList, queueList, waitingGames, gameList, gameResults, status, settings, connect, disconnect, updateSettings }
}


