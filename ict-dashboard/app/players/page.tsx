"use client"

import { useServer } from "@/components/server-provider"
import { SiteHeader } from "@/components/site-header"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Badge } from "@/components/ui/badge"
import { PlayerDataTable } from "@/components/playter-data-table"

export default function PlayersPage() {
  const { playerList } = useServer()

  return (
    <div className="flex flex-col min-h-screen">
      <SiteHeader name="Player Directory" />

      {/* Main Content Container */}
      <main className="flex-1 p-6 lg:p-8">
        <div className="mx-auto max-w-6xl space-y-6">

          {/* Page Header */}
          <div className="flex items-center justify-between">
            <div>
              <h2 className="text-2xl font-bold tracking-tight">Player Management</h2>
              <p className="text-muted-foreground">
                View, search, and monitor all currently connected players.
              </p>
            </div>
          </div>

          {/* Table Card */}
          <Card>
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-6">
              <div className="space-y-1.5">
                <CardTitle>Connected Players</CardTitle>
                <CardDescription>
                  A complete directory of all users on the server.
                </CardDescription>
              </div>
              <Badge variant="secondary" className="text-sm px-3 py-1 font-medium">
                {playerList?.length || 0} Total
              </Badge>
            </CardHeader>
            <CardContent>
              <PlayerDataTable data={playerList || []} />
            </CardContent>
          </Card>

        </div>
      </main>
    </div>
  )
}