"use client"

import * as React from "react"
import {
    flexRender,
    getCoreRowModel,
    getPaginationRowModel,
    useReactTable,
    type ColumnDef,
} from "@tanstack/react-table"
import { ChevronLeftIcon, ChevronRightIcon } from "lucide-react"

import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import {
    Table,
    TableBody,
    TableCell,
    TableHead,
    TableHeader,
    TableRow,
} from "@/components/ui/table"
import { LobbyInfo, QueueInfo } from "@/hooks/useDashboard"

// --- 1. LOBBY TABLE COLUMNS ---
const lobbyColumns: ColumnDef<LobbyInfo>[] = [
    {
        accessorKey: "id",
        header: "Lobby ID",
        cell: ({ row }) => {
            const id = row.getValue("id") as string
            return <div className="font-mono text-xs text-muted-foreground">{id.split("-")[0]}...</div>
        },
    },
    {
        accessorKey: "owner",
        header: "Owner ID",
        cell: ({ row }) => {
            const owner = row.getValue("owner") as string
            return <div className="font-mono text-xs">{owner.split("-")[0]}...</div>
        },
    },
    {
        accessorKey: "players",
        header: "Players",
        cell: ({ row }) => {
            const players = row.getValue("players") as string[]
            return (
                <div className="flex items-center gap-2">
                    <Badge variant="secondary">{players.length}/4</Badge>
                </div>
            )
        },
    },
    {
        accessorKey: "status",
        header: "Status",
        cell: ({ row }) => {
            const status = row.getValue("status") as any

            let statusText = "Unknown"
            let colorClass = "bg-gray-500/10 text-gray-500 border-gray-500/20"

            if (typeof status === "string") {
                statusText = status
            } else if (status) {
                if ("InQueue" in status) {
                    statusText = "In Queue"
                    colorClass = "bg-blue-500/10 text-blue-500 border-blue-500/20"
                } else if ("NeedConfirmation" in status) {
                    statusText = "Need Confirmation"
                    colorClass = "bg-yellow-500/10 text-yellow-500 border-yellow-500/20"
                } else if ("InGame" in status) {
                    statusText = "In Game"
                    colorClass = "bg-red-500/10 text-red-500 border-red-500/20"
                }
            }

            return (
                <Badge variant="outline" className={colorClass}>
                    {statusText}
                </Badge>
            )
        },
    },
]

// --- 2. QUEUE TABLE COLUMNS ---
const queueColumns: ColumnDef<QueueInfo>[] = [
    {
        accessorKey: "lobby_id",
        header: "Lobby ID",
        cell: ({ row }) => {
            const id = row.getValue("lobby_id") as string
            return <div className="font-mono text-xs text-muted-foreground">{id.split("-")[0]}...</div>
        },
    },
    {
        accessorKey: "avg_mmr",
        header: "Avg MMR",
        cell: ({ row }) => <div className="font-mono">{Math.round(row.getValue("avg_mmr") as number)}</div>,
    },
    {
        accessorKey: "player_count",
        header: "Group Size",
        cell: ({ row }) => <div>{row.getValue("player_count")}</div>,
    },
    {
        accessorKey: "queue_seconds",
        header: "Time in Queue",
        cell: ({ row }) => {
            const seconds = row.getValue("queue_seconds") as number
            const mins = Math.floor(seconds / 60)
            const secs = seconds % 60

            // Turn text red if they have been waiting longer than 3 minutes
            const isLongWait = seconds > 180

            return (
                <div className={`font-mono ${isLongWait ? 'text-red-500 font-semibold' : ''}`}>
                    {mins > 0 ? `${mins}m ` : ''}{secs}s
                </div>
            )
        },
    },
]

// --- 3. REUSABLE BASE TABLE COMPONENT ---
function BaseTable<TData, TValue>({
    columns,
    data,
    emptyMessage
}: {
    columns: ColumnDef<TData, TValue>[],
    data: TData[],
    emptyMessage: string
}) {
    const [pagination, setPagination] = React.useState({ pageIndex: 0, pageSize: 5 })

    const table = useReactTable({
        data,
        columns,
        state: { pagination },
        onPaginationChange: setPagination,
        getCoreRowModel: getCoreRowModel(),
        getPaginationRowModel: getPaginationRowModel(),
    })

    return (
        <div className="w-full flex flex-col gap-4">
            <div className="rounded-md border overflow-hidden">
                <Table>
                    <TableHeader className="bg-muted/50">
                        {table.getHeaderGroups().map((headerGroup) => (
                            <TableRow key={headerGroup.id}>
                                {headerGroup.headers.map((header) => (
                                    <TableHead key={header.id}>
                                        {header.isPlaceholder ? null : flexRender(header.column.columnDef.header, header.getContext())}
                                    </TableHead>
                                ))}
                            </TableRow>
                        ))}
                    </TableHeader>
                    <TableBody>
                        {table.getRowModel().rows?.length ? (
                            table.getRowModel().rows.map((row) => (
                                <TableRow key={row.id}>
                                    {row.getVisibleCells().map((cell) => (
                                        <TableCell key={cell.id}>
                                            {flexRender(cell.column.columnDef.cell, cell.getContext())}
                                        </TableCell>
                                    ))}
                                </TableRow>
                            ))
                        ) : (
                            <TableRow>
                                <TableCell colSpan={columns.length} className="h-24 text-center text-muted-foreground">
                                    {emptyMessage}
                                </TableCell>
                            </TableRow>
                        )}
                    </TableBody>
                </Table>
            </div>

            {/* Minimal Pagination */}
            {table.getPageCount() > 1 && (
                <div className="flex items-center justify-end gap-2">
                    <Button variant="outline" size="sm" onClick={() => table.previousPage()} disabled={!table.getCanPreviousPage()}>
                        <ChevronLeftIcon className="h-4 w-4" />
                    </Button>
                    <span className="text-sm text-muted-foreground mx-2">
                        Page {table.getState().pagination.pageIndex + 1} of {table.getPageCount()}
                    </span>
                    <Button variant="outline" size="sm" onClick={() => table.nextPage()} disabled={!table.getCanNextPage()}>
                        <ChevronRightIcon className="h-4 w-4" />
                    </Button>
                </div>
            )}
        </div>
    )
}

// --- 4. EXPORTED COMPONENTS ---
export function LobbyDataTable({ data }: { data: LobbyInfo[] }) {
    return <BaseTable columns={lobbyColumns} data={data} emptyMessage="No active lobbies found." />
}

export function QueueDataTable({ data }: { data: QueueInfo[] }) {
    return <BaseTable columns={queueColumns} data={data} emptyMessage="No lobbies currently in queue." />
}