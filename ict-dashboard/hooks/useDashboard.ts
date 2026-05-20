import { useCallback, useEffect, useRef, useState } from "react"

//{"type":"Snapshot","snapshot":{"total_player":49,"connected_players":0,"lobby_number":0,"game_number":0}}
export interface DashboardSnapshot {
    total_player: number
    connected_players: number
    lobby_number: number
    game_number: number
}

//{"type":"PlayerList","players":[{"id":"7f578f49-5678-4ce1-925b-23369748d551","name":"amuck-glove","mmr":1000.0,"rank":0,"div":4,"points":0,"status":"Idle","lobby":null,"skills":["nonworkers","ashcans","hatchet","hillier"]}]}
//{ "type": "PlayerList", "players": [{ "id": "ee053c67-3a92-49af-a908-3acf2dd4d46d", "name": "trite-cub", "mmr": 1000.0, "rank": 0, "div": 4, "points": 0, "status": { "NeedConfirmation": { "game_id": "985225bc-80c2-4af2-ab19-86743afeff2d" } }, "lobby": "63ac3a98-a9d8-4d07-8f58-bcc195931b77", "skills": ["documented", "soupcon", "outgives", "mantelpieces"] }] }
export interface PlayerInfo {
    id: string
    name: string
    mmr: number
    rank: number
    div: number
    points: number
    // status: "Idle" | "InQueue" | "NeedConfirmation" | "InGame"
    status: "Idle" | "InQueue" | { NeedConfirmation: { game_id: string } } | { InGame: { game_id: string } }
    lobby: string | null
    skills: string[]
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
    const [status, setStatus] = useState<ConnectionStatus>("disconnected")
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

    return { data, playerList, lobbyList, queueList, waitingGames, gameList, status, connect, disconnect }
}

