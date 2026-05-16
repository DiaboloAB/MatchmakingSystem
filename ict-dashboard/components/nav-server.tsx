"use client"

import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu"
import {
  SidebarGroup,
  SidebarGroupLabel,
  SidebarMenu,
  SidebarMenuAction,
  SidebarMenuButton,
  SidebarMenuItem,
  useSidebar,
} from "@/components/ui/sidebar"
import { MoreHorizontalIcon, FolderIcon, ShareIcon, Trash2Icon, RefreshCw } from "lucide-react"
import { useServer } from "./server-provider"
import { Input } from "./ui/input"
import { Badge } from "@/components/ui/badge"


import {
  CircleCheckIcon,
  CircleMinusIcon,
  AlertCircleIcon,
  Loader2Icon
} from "lucide-react"
import { Button } from "./ui/button"

const statusConfig = {
  connected: {
    icon: CircleCheckIcon,
    className: "fill-green-500 dark:fill-green-400 text-green-500",
  },
  disconnected: {
    icon: CircleMinusIcon,
    className: "fill-slate-500 dark:fill-slate-400 text-slate-500",
  },
  connecting: {
    icon: Loader2Icon,
    className: "animate-spin text-blue-500",
  },
  error: {
    icon: AlertCircleIcon,
    className: "fill-red-500 dark:fill-red-400 text-red-500",
  },
} as const; // Typescript safety

export function StatusBadge({ status }: { status: keyof typeof statusConfig }) {
  // 2. Grab the correct configuration based on the current status
  const { icon: Icon, className } = statusConfig[status];

  return (
    <Badge variant="outline" className="px-1.5 text-muted-foreground gap-1.5">
      <Icon className={`h-4 w-4 ${className}`} />
      <span className="capitalize">{status}</span>
    </Badge>
  )
}

export function NavServer() {
  const { isMobile } = useSidebar()
  const { data, status, port, setPort, connect } = useServer()

  return (
    <SidebarGroup className="group-data-[collapsible=icon]:hidden">
      <SidebarGroupLabel>Server <span className="pl-2"><StatusBadge status={status} /></span></SidebarGroupLabel>
      <SidebarMenu>

        <SidebarMenuItem>
          <div className="flex items-center gap-2 px-2 pb-2 pt-1">
            <Input
              type="number"
              value={port}
              onChange={(e) => setPort(Number(e.target.value))}
              placeholder="12345"
              className="h-8 w-full text-xs"
            />
            <Button
              variant="outline"
              size="icon"
              className="h-8 w-8 shrink-0"
              onClick={connect}
              disabled={status === "connecting"}
            >
              <RefreshCw
                className={`h-4 w-4 ${status === "connecting" ? "animate-spin text-blue-500" : ""}`}
              />
              <span className="sr-only">Reconnect</span>
            </Button>
          </div>
        </SidebarMenuItem>
      </SidebarMenu>
    </SidebarGroup >
  )
}
