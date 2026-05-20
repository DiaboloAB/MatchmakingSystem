"use client"

import { LobbyDataTable, QueueDataTable } from "@/components/lobby-queue-table"
import { useServer } from "@/components/server-provider"
import { SiteHeader } from "@/components/site-header"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Badge } from "@/components/ui/badge"

export default function LobbiesPage() {
  const { lobbyList, queueList } = useServer()

  return (
    <div className="flex flex-col min-h-screen">
      <SiteHeader name="Matchmaking Overview" />

      {/* Main Content Container */}
      <main className="flex-1 p-6 lg:p-8">
        <div className="mx-auto max-w-6xl space-y-6">

          <div className="flex items-center justify-between">
            <div>
              <h2 className="text-2xl font-bold tracking-tight">Lobby & Queue Management</h2>
              <p className="text-muted-foreground">
                Monitor active matchmaking queues and server lobbies.
              </p>
            </div>
          </div>

          <Tabs defaultValue="queues" className="space-y-6">
            <TabsList className="grid w-full grid-cols-2 lg:w-[400px]">
              <TabsTrigger value="queues" className="flex items-center gap-2">
                Active Queues
                <Badge variant="secondary" className="rounded-full px-2 py-0.5 text-xs font-normal">
                  {queueList?.length || 0}
                </Badge>
              </TabsTrigger>
              <TabsTrigger value="lobbies" className="flex items-center gap-2">
                All Lobbies
                <Badge variant="secondary" className="rounded-full px-2 py-0.5 text-xs font-normal">
                  {lobbyList?.length || 0}
                </Badge>
              </TabsTrigger>
            </TabsList>

            <TabsContent value="queues" className="space-y-4 outline-none">
              <Card>
                <CardHeader>
                  <CardTitle>Matchmaking Queues</CardTitle>
                  <CardDescription>
                    Lobbies that are currently waiting for a match.
                  </CardDescription>
                </CardHeader>
                <CardContent>
                  <QueueDataTable data={queueList || []} />
                </CardContent>
              </Card>
            </TabsContent>

            <TabsContent value="lobbies" className="space-y-4 outline-none">
              <Card>
                <CardHeader>
                  <CardTitle>Registered Lobbies</CardTitle>
                  <CardDescription>
                    All active lobbies currently connected to the server.
                  </CardDescription>
                </CardHeader>
                <CardContent>
                  <LobbyDataTable data={lobbyList || []} />
                </CardContent>
              </Card>
            </TabsContent>
          </Tabs>

        </div>
      </main>
    </div>
  )
}