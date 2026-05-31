"use client"

import { useServer } from "./server-provider"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Badge } from "@/components/ui/badge"

export function TopPlayersCard() {
    const { playerList } = useServer()

    const topPlayers = [...(playerList || [])]
        .sort((a, b) => b.mmr - a.mmr)
        .slice(0, 10)

    return (
        <Card className="flex flex-col h-full">
            <CardHeader>
                <CardTitle>Top 10 Players</CardTitle>
                <CardDescription>Highest MMR on the server</CardDescription>
            </CardHeader>
            <CardContent className="flex-1 overflow-y-auto">
                <div className="space-y-4">
                    {topPlayers.length > 0 ? (
                        topPlayers.map((player, index) => {
                            const winCount = player.wins?.length || 0;
                            const lossCount = player.losses?.length || 0;

                            return (
                                <div key={player.id} className="flex items-center justify-between border-b pb-2 last:border-0 last:pb-0">
                                    <div className="flex items-center gap-3">
                                        <span className="text-muted-foreground font-mono text-sm w-4">
                                            {index + 1}.
                                        </span>
                                        <div className="font-medium text-sm truncate max-w-[120px]">
                                            {player.name}
                                        </div>
                                    </div>
                                    <div className="flex items-center gap-2">
                                        <span className="text-xs text-muted-foreground font-mono hidden sm:inline-block">
                                            {winCount}W - {lossCount}L
                                        </span>
                                        <Badge variant="outline" className="text-xs">{player.debug_rank}</Badge>
                                        <Badge variant="secondary" className="font-mono">
                                            {Math.round(player.mmr)} MMR
                                        </Badge>
                                    </div>
                                </div>
                            )
                        })
                    ) : (
                        <div className="text-sm text-muted-foreground text-center py-8">
                            No player data available.
                        </div>
                    )}
                </div>
            </CardContent>
        </Card>
    )
}