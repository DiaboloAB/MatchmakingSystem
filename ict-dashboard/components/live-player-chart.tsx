"use client"

import * as React from "react"
import { Area, AreaChart, CartesianGrid, XAxis, YAxis } from "recharts"

import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"
import {
  ChartContainer,
  ChartTooltip,
  ChartTooltipContent,
  type ChartConfig,
} from "@/components/ui/chart"
import { useServer } from "./server-provider"

const chartConfig = {
  players: {
    label: "Online Players",
    color: "hsl(var(--chart-1))",
  },
} satisfies ChartConfig

export function LivePlayerChart() {
  const { data, status } = useServer()

  const [chartData, setChartData] = React.useState<{ time: string; players: number }[]>([])

  React.useEffect(() => {
    if (status !== "connected") return

    const recordDataPoint = () => {
      const now = new Date()
      const timeString = now.toLocaleTimeString([], {
        hour: '2-digit',
        minute: '2-digit',
        second: '2-digit'
      })

      const currentPlayers = data?.connected_players || 0

      setChartData((prev) => {
        const newData = [...prev, { time: timeString, players: currentPlayers }]
        if (newData.length > 20) {
          newData.shift()
        }
        return newData
      })
    }

    recordDataPoint()
    const interval = setInterval(recordDataPoint, 5000)

    return () => clearInterval(interval)
  }, [data?.connected_players, status])

  return (
    <Card className="flex flex-col h-full">
      <CardHeader>
        <CardTitle>Live Server Traffic</CardTitle>
        <CardDescription>
          {status === "connected"
            ? "Tracking connected players over the current session."
            : "Waiting for connection..."}
        </CardDescription>
      </CardHeader>
      <CardContent className="flex-1 px-2 pt-4 sm:px-6 sm:pt-6">
        {chartData.length > 0 ? (
          <ChartContainer
            config={chartConfig}
            className="aspect-auto h-[250px] w-full"
          >
            <AreaChart data={chartData} margin={{ top: 10, right: 10, left: -20, bottom: 0 }}>
              <defs>
                <linearGradient id="fillPlayers" x1="0" y1="0" x2="0" y2="1">
                  <stop
                    offset="5%"
                    stopColor="var(--color-players)"
                    stopOpacity={0.8}
                  />
                  <stop
                    offset="95%"
                    stopColor="var(--color-players)"
                    stopOpacity={0.1}
                  />
                </linearGradient>
              </defs>
              <CartesianGrid vertical={false} strokeDasharray="3 3" />
              <XAxis
                dataKey="time"
                tickLine={false}
                axisLine={false}
                tickMargin={8}
                minTickGap={32}
              />
              <YAxis
                tickLine={false}
                axisLine={false}
                tickMargin={8}
                allowDecimals={false}
              />
              <ChartTooltip
                cursor={false}
                content={<ChartTooltipContent indicator="dot" />}
              />
              <Area
                dataKey="players"
                type="stepAfter"
                fill="url(#fillPlayers)"
                stroke="var(--color-players)"
                strokeWidth={2}
                isAnimationActive={false}
              />
            </AreaChart>
          </ChartContainer>
        ) : (
          <div className="flex h-[250px] items-center justify-center text-sm text-muted-foreground">
            Gathering telemetry...
          </div>
        )}
      </CardContent>
    </Card>
  )
}