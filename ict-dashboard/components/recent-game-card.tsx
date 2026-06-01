"use client"

import { useServer } from "./server-provider"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Badge } from "@/components/ui/badge"
import { TrophyIcon, SwordsIcon } from "lucide-react"

export function RecentGameCard() {
    const { gameResults, playerList } = useServer()

    const recentGames = [...(gameResults || [])].reverse().slice(0, 15)

    const getPlayerName = (id: string) => {
        const player = playerList?.find((p) => p.id === id)
        return player ? player.name : id.split("-")[0]
    }

    return (
        <Card className="flex flex-col h-full">
            <CardHeader>
                <CardTitle className="flex items-center gap-2">
                    <SwordsIcon className="size-5" />
                    Recent Matches
                </CardTitle>
                <CardDescription>Live feed of completed games</CardDescription>
            </CardHeader>
            <CardContent className="flex-1 overflow-y-auto">
                <div className="space-y-4">
                    {recentGames.length > 0 ? (
                        recentGames.map((game) => {
                            const team1Names = game.team1.map(getPlayerName).join(", ")
                            const team2Names = game.team2.map(getPlayerName).join(", ")

                            const t1Won = game.winner === 1
                            const t2Won = game.winner === 2

                            return (
                                <div key={game.id} className="flex flex-col gap-2 border-b pb-3 last:border-0 last:pb-0">
                                    <div className="flex items-center justify-between">
                                        <span className="text-xs text-muted-foreground font-mono">{game.time}</span>
                                        <span className="text-xs text-muted-foreground font-mono" title={game.id}>
                                            ID: {game.id.split("-")[0]}
                                        </span>
                                    </div>

                                    <div className="flex items-center justify-between gap-4">
                                        {/* Team 1 */}
                                        <div className={`flex-1 text-sm truncate ${t1Won ? "font-semibold" : "text-muted-foreground"}`}>
                                            {team1Names}
                                        </div>

                                        {/* Winner Badge */}
                                        <Badge variant={t1Won ? "default" : t2Won ? "destructive" : "secondary"} className="shrink-0">
                                            {t1Won ? "Team 1" : "Team 2"} <TrophyIcon className="ml-1 size-3" />
                                        </Badge>

                                        {/* Team 2 */}
                                        <div className={`flex-1 text-right text-sm truncate ${t2Won ? "font-semibold" : "text-muted-foreground"}`}>
                                            {team2Names}
                                        </div>
                                    </div>
                                </div>
                            )
                        })
                    ) : (
                        <div className="text-sm text-muted-foreground text-center py-8">
                            No recent matches.
                        </div>
                    )}
                </div>
            </CardContent>
        </Card>
    )
}