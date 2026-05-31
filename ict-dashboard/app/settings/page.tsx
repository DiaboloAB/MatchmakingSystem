"use client"

import React, { useEffect, useState } from "react"
import { useServer } from "@/components/server-provider"
import { SiteHeader } from "@/components/site-header"
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from "@/components/ui/card"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Button } from "@/components/ui/button"
import { Badge } from "@/components/ui/badge"
import { toast } from "sonner"
import type { AppSettings } from "@/hooks/useDashboard"

export default function SettingsPage() {
  const { settings, status, updateSettings } = useServer()

  const [formData, setFormData] = useState<AppSettings | null>(null)

  useEffect(() => {
    if (settings && !formData) {
      setFormData(settings)
    }
  }, [settings, formData])

  const handleInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const { name, value } = e.target
    if (formData) {
      setFormData({
        ...formData,
        [name]: parseFloat(value) || 0
      })
    }
  }

  const handleSave = () => {
    if (!formData) return

    try {
      updateSettings(formData)
      toast.success("Settings updated successfully")
    } catch (error) {
      toast.error("Failed to update settings")
    }
  }

  return (
    <div className="flex flex-col min-h-screen">
      <SiteHeader name="Server Settings" />

      <main className="flex-1 p-6 lg:p-8">
        <div className="mx-auto max-w-3xl space-y-6">
          <div className="flex items-center justify-between">
            <div>
              <h2 className="text-2xl font-bold tracking-tight">Configuration</h2>
              <p className="text-muted-foreground">
                Manage matchmaking and simulation parameters.
              </p>
            </div>
            <Badge variant={status === "connected" ? "default" : "destructive"} className="uppercase">
              {status}
            </Badge>
          </div>

          <Card>
            <CardHeader>
              <CardTitle>Matchmaking Parameters</CardTitle>
              <CardDescription>
                Changes applied here will be sent to the live server.
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-6">
              {!formData ? (
                <div className="text-sm text-muted-foreground py-8 text-center">
                  Waiting for server settings...
                </div>
              ) : (
                <div className="grid grid-cols-1 sm:grid-cols-2 gap-6">

                  <div className="space-y-2">
                    <Label htmlFor="lobby_capacity">Lobby Capacity</Label>
                    <Input
                      id="lobby_capacity"
                      name="lobby_capacity"
                      type="number"
                      value={formData.lobby_capacity}
                      onChange={handleInputChange}
                    />
                    <p className="text-[0.8rem] text-muted-foreground">Max players allowed in a single lobby.</p>
                  </div>

                  <div className="space-y-2">
                    <Label htmlFor="team_size">Team Size</Label>
                    <Input
                      id="team_size"
                      name="team_size"
                      type="number"
                      value={formData.team_size}
                      onChange={handleInputChange}
                    />
                    <p className="text-[0.8rem] text-muted-foreground">Number of players per team in a match.</p>
                  </div>

                  <div className="space-y-2">
                    <Label htmlFor="simulation_speed">Simulation Speed</Label>
                    <Input
                      id="simulation_speed"
                      name="simulation_speed"
                      type="number"
                      step="0.1"
                      value={formData.simulation_speed}
                      onChange={handleInputChange}
                    />
                    <p className="text-[0.8rem] text-muted-foreground">Game tick speed multiplier.</p>
                  </div>

                  <div className="space-y-2">
                    <Label htmlFor="confirmation_time">Confirmation Time (s)</Label>
                    <Input
                      id="confirmation_time"
                      name="confirmation_time"
                      type="number"
                      value={formData.confirmation_time}
                      onChange={handleInputChange}
                    />
                    <p className="text-[0.8rem] text-muted-foreground">Seconds allowed for players to accept a match.</p>
                  </div>

                </div>
              )}
            </CardContent>
            <CardFooter className="flex justify-end border-t p-6">
              <Button
                onClick={handleSave}
                disabled={!formData || status !== "connected"}
              >
                Save Changes
              </Button>
            </CardFooter>
          </Card>
        </div>
      </main>
    </div>
  )
}