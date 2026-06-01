"use client"

import * as React from "react"
import { Bar, BarChart, CartesianGrid, XAxis, YAxis, Tooltip } from "recharts"
import { useServer } from "./server-provider"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { ChartContainer, ChartTooltipContent, type ChartConfig } from "@/components/ui/chart"

const chartConfig = {
    players: {
        label: "Players",
        color: "var(--primary)",
    },
} satisfies ChartConfig

const RANK_ORDER = [
    "Unranked",
    "Iron",
    "Bronze",
    "Silver",
    "Gold",
    "Platinum",
    "Emerald",
    "Diamond",
    "Master",
    "Grandmaster",
    "Challenger"
]

export function RankDistributionChart() {
    const { playerList } = useServer()

    const distributionData = React.useMemo(() => {
        if (!playerList) return []

        const counts: Record<string, number> = {}
        RANK_ORDER.forEach(rank => {
            counts[rank] = 0
        })

        playerList.forEach((player) => {
            const key = player.debug_rank || "Unranked"

            if (counts[key] === undefined) {
                counts[key] = 0
            }

            counts[key] += 1
        })

        const orderedData = RANK_ORDER.map(name => ({
            name,
            players: counts[name]
        }))

        const knownRanks = new Set(RANK_ORDER)
        Object.keys(counts).forEach(key => {
            if (!knownRanks.has(key)) {
                orderedData.push({ name: key, players: counts[key] })
            }
        })

        return orderedData
    }, [playerList])

    return (
        <Card className="h-full flex flex-col">
            <CardHeader>
                <CardTitle>Rank Distribution</CardTitle>
                <CardDescription>Current player population by rank</CardDescription>
            </CardHeader>
            <CardContent className="flex-1">
                {distributionData.length > 0 ? (
                    <ChartContainer config={chartConfig} className="aspect-auto h-full min-h-[250px] w-full">
                        <BarChart data={distributionData} margin={{ top: 10, right: 10, left: -20, bottom: 0 }}>
                            <CartesianGrid vertical={false} strokeDasharray="3 3" />
                            <XAxis
                                dataKey="name"
                                tickLine={false}
                                axisLine={false}
                                tickMargin={8}
                            />
                            <YAxis
                                tickLine={false}
                                axisLine={false}
                                tickMargin={8}
                                allowDecimals={false}
                            />
                            <Tooltip cursor={{ fill: 'var(--muted)' }} content={<ChartTooltipContent />} />
                            <Bar
                                dataKey="players"
                                fill="var(--color-players)"
                                radius={[4, 4, 0, 0]}
                            />
                        </BarChart>
                    </ChartContainer>
                ) : (
                    <div className="flex h-[250px] items-center justify-center text-sm text-muted-foreground">
                        Not enough data to display.
                    </div>
                )}
            </CardContent>
        </Card>
    )
}