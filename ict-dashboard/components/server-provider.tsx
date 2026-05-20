"use client"

import React, { createContext, useContext, useState } from "react";
import { useDashboard, ConnectionStatus } from "@/hooks/useDashboard";

import type { DashboardSnapshot, GameInfo, LobbyInfo, PlayerInfo, QueueInfo } from "@/hooks/useDashboard";

interface ServerContextType {
    port: number;
    setPort: React.Dispatch<React.SetStateAction<number>>;
    data: DashboardSnapshot | null;
    playerList: PlayerInfo[];
    lobbyList: LobbyInfo[];
    queueList: QueueInfo[];
    waitingGames: GameInfo[];
    gameList: GameInfo[];
    status: ConnectionStatus;
    connect: () => void;
    disconnect: () => void;
}

const ServerContext = createContext<ServerContextType | undefined>(undefined);

export default function ServerProvider({
    children,
}: Readonly<{
    children: React.ReactNode
}>) {
    const [port, setPort] = useState(12345)

    const { data, playerList, lobbyList, queueList, waitingGames, gameList, status, connect, disconnect } = useDashboard({ port })

    return (
        <ServerContext.Provider value={{ port, setPort, data, playerList, lobbyList, queueList, waitingGames, gameList, status, connect, disconnect }}>
            {children}
        </ServerContext.Provider>
    )
}
export function useServer() {
    const context = useContext(ServerContext);

    if (context === undefined) {
        throw new Error("useServer must be used within a ServerProvider");
    }

    return context;
}