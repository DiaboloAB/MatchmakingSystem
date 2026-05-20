"use client"

import { useServer } from "@/components/server-provider"
import { SiteHeader } from "@/components/site-header"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Badge } from "@/components/ui/badge"
import { WaitingGamesTable, OngoingGamesTable } from "@/components/games-tables"

export default function GamesPage() {
  const { waitingGames, gameList, status } = useServer()

  return (
    <div className="flex flex-col min-h-screen">
      <SiteHeader name="Matches Overview" />

      {/* Main Content Container */}
      <main className="flex-1 p-6 lg:p-8">
        <div className="mx-auto max-w-6xl space-y-6">

          <div className="flex items-center justify-between">
            <div>
              <h2 className="text-2xl font-bold tracking-tight">Game Servers</h2>
              <p className="text-muted-foreground">
                Monitor active matches and lobby confirmations.
              </p>
            </div>
          </div>

          <Tabs defaultValue="ongoing" className="space-y-6">
            <TabsList className="grid w-full grid-cols-2 lg:w-[450px]">
              <TabsTrigger value="ongoing" className="flex items-center gap-2">
                Ongoing Matches
                <Badge variant="secondary" className="rounded-full px-2 py-0.5 text-xs font-normal">
                  {gameList?.length || 0}
                </Badge>
              </TabsTrigger>
              <TabsTrigger value="waiting" className="flex items-center gap-2">
                Waiting Confirmation
                <Badge variant="secondary" className="rounded-full px-2 py-0.5 text-xs font-normal">
                  {waitingGames?.length || 0}
                </Badge>
              </TabsTrigger>
            </TabsList>

            <TabsContent value="ongoing" className="space-y-4 outline-none">
              <Card>
                <CardHeader>
                  <CardTitle>Active Games</CardTitle>
                  <CardDescription>
                    Matches that are currently being played on the server.
                  </CardDescription>
                </CardHeader>
                <CardContent>
                  <OngoingGamesTable data={gameList || []} />
                </CardContent>
              </Card>
            </TabsContent>

            <TabsContent value="waiting" className="space-y-4 outline-none">
              <Card>
                <CardHeader>
                  <CardTitle>Pending Confirmations</CardTitle>
                  <CardDescription>
                    Matchmaking is complete, waiting for all players to accept.
                  </CardDescription>
                </CardHeader>
                <CardContent>
                  <WaitingGamesTable data={waitingGames || []} />
                </CardContent>
              </Card>
            </TabsContent>
          </Tabs>

        </div>
      </main>
    </div>
  )
}