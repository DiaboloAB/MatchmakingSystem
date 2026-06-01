"use client"

import { Badge } from "@/components/ui/badge"
import {
  Card,
  CardAction,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"
import { UsersIcon, SwordsIcon, ClockIcon, ActivityIcon } from "lucide-react"
import { useServer } from "@/components/server-provider"

export function SectionCards() {
  const { data, queueList, playerList } = useServer()

  // --- Calculations ---
  // 1. Players currently in queue
  const playersInQueue = queueList?.reduce((acc, curr) => acc + curr.player_count, 0) || 0

  // 2. Average Queue Time
  const avgQueueSeconds = queueList?.length
    ? Math.floor(queueList.reduce((acc, curr) => acc + curr.queue_seconds, 0) / queueList.length)
    : 0
  const avgQueueText = avgQueueSeconds > 60
    ? `${Math.floor(avgQueueSeconds / 60)}m ${avgQueueSeconds % 60}s`
    : `${avgQueueSeconds}s`

  // 3. Online Players percentage
  const onlinePercentage = data?.total_player
    ? Math.round((data.connected_players / data.total_player) * 100)
    : 0

  //{"type":"Snapshot","snapshot":{"total_player":96,"total_finished_game":89,"connected_players":13,"lobby_number":12,"game_number":1,"queueing_lobbies":2,"waiting_games":3,"ongoing_games":1}}


  return (
    <div className="grid grid-cols-1 gap-4 px-4 *:data-[slot=card]:bg-gradient-to-t *:data-[slot=card]:from-primary/5 *:data-[slot=card]:to-card *:data-[slot=card]:shadow-xs lg:px-6 @xl/main:grid-cols-2 @5xl/main:grid-cols-4 dark:*:data-[slot=card]:bg-card">

      {/* CARD 1: Server Population */}
      <Card className="@container/card">
        <CardHeader>
          <CardDescription>Online Players</CardDescription>
          <CardTitle className="text-2xl font-semibold tabular-nums @[250px]/card:text-3xl">
            {data ? data.connected_players : 0}
          </CardTitle>
          <CardAction>
            <Badge variant="outline" className="font-mono">
              <UsersIcon className="mr-1 size-3" />
              {data ? data.total_player : 0} Total
            </Badge>
          </CardAction>
        </CardHeader>
        <CardFooter className="flex-col items-start gap-1.5 text-sm">
          <div className="text-muted-foreground">
            {onlinePercentage}% of userbase online
          </div>
        </CardFooter>
      </Card>

      {/* CARD 3: Queue Volume */}
      <Card className="@container/card">
        <CardHeader>
          <CardDescription>Queue Volume</CardDescription>
          <CardTitle className="text-2xl font-semibold tabular-nums @[250px]/card:text-3xl">
            {data?.queueing_lobbies || 0}
          </CardTitle>
          <CardAction>
            <Badge variant="outline">
              <ActivityIcon className="mr-1 size-3" />
              Matchmaking
            </Badge>
          </CardAction>
        </CardHeader>
        <CardFooter className="flex-col items-start gap-1.5 text-sm">
          <div className="text-muted-foreground">
            total lobbies: {data?.lobby_number || 0}
          </div>
        </CardFooter>
      </Card>

      {/* CARD 2: Matchmaking Health */}
      <Card className="@container/card">
        <CardHeader>
          <CardDescription>Avg. Wait Time</CardDescription>
          <CardTitle className="text-2xl font-semibold tabular-nums @[250px]/card:text-3xl">
            {data ? data.average_queue_time.toPrecision(3) : 0} seconds
          </CardTitle>
          <CardAction>
            <Badge variant="outline" className={avgQueueSeconds > 120 ? "text-yellow-500" : "text-green-500"}>
              <ClockIcon className="mr-1 size-3" />
              Live
            </Badge>
          </CardAction>
        </CardHeader>
        <CardFooter className="flex-col items-start gap-1.5 text-sm">
          <div className="text-muted-foreground">
            game success rate: {data ? (data.successful_match_rate * 100).toPrecision(3) : 0}%
          </div>
        </CardFooter>
      </Card>


      {/* CARD 4: Active Matches */}
      <Card className="@container/card">
        <CardHeader>
          <CardDescription>Ongoing Games</CardDescription>
          <CardTitle className="text-2xl font-semibold tabular-nums @[250px]/card:text-3xl">
            {data ? data.ongoing_games : 0}
          </CardTitle>
          <CardAction>
            <Badge variant="outline" className="text-red-500">
              <SwordsIcon className="mr-1 size-3" />
              In Progress
            </Badge>
          </CardAction>
        </CardHeader>
        <CardFooter className="flex-col items-start gap-1.5 text-sm">
          <div className="text-muted-foreground">
            total finished: {data?.total_finished_game || 0}
          </div>
        </CardFooter>
      </Card>

    </div>
  )
}