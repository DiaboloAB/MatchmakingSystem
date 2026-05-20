"use client"

import * as React from "react"
import {
    flexRender,
    getCoreRowModel,
    getPaginationRowModel,
    useReactTable,
    type ColumnDef,
} from "@tanstack/react-table"
import { ChevronLeftIcon, ChevronRightIcon, ClockIcon, CheckCircle2Icon, HourglassIcon } from "lucide-react"

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
import { GameInfo } from "@/hooks/useDashboard"

// --- Helper for Time Formatting ---
const formatTime = (seconds: number) => {
    const mins = Math.floor(seconds / 60)
    const secs = seconds % 60
    return mins > 0 ? `${mins}m ${secs}s` : `${secs}s`
}

// --- 1. WAITING GAMES COLUMNS ---
const waitingColumns: ColumnDef<GameInfo>[] = [
    {
        accessorKey: "id",
        header: "Game ID",
        cell: ({ row }) => {
            const id = row.getValue("id") as string
            return <div className="font-mono text-xs text-muted-foreground">{id.split("-")[0]}...</div>
        },
    },
    {
        id: "format",
        header: "Format",
        cell: ({ row }) => {
            const team1 = row.original.team1.length
            const team2 = row.original.team2.length
            return <Badge variant="secondary">{team1} v {team2}</Badge>
        },
    },
    {
        id: "confirmation",
        header: "Status",
        cell: ({ row }) => {
            const totalPlayers = row.original.team1.length + row.original.team2.length
            const confirmedCount = row.original.confirmed.length
            const isFullyConfirmed = confirmedCount === totalPlayers

            return (
                <Badge variant="outline" className={isFullyConfirmed ? "bg-green-500/10 text-green-500 border-green-500/20" : "bg-yellow-500/10 text-yellow-500 border-yellow-500/20"}>
                    {isFullyConfirmed ? <CheckCircle2Icon className="mr-1 size-3" /> : <HourglassIcon className="mr-1 size-3" />}
                    {confirmedCount} / {totalPlayers} Confirmed
                </Badge>
            )
        },
    },
    {
        accessorKey: "elapsed_seconds",
        header: "Time Waiting",
        cell: ({ row }) => {
            const seconds = row.getValue("elapsed_seconds") as number
            // Highlight in red if waiting for more than 30 seconds
            return (
                <div className={`flex items-center gap-2 font-mono ${seconds > 30 ? 'text-red-500' : ''}`}>
                    <ClockIcon className="size-3" />
                    {formatTime(seconds)}
                </div>
            )
        },
    },
]

// --- 2. ONGOING GAMES COLUMNS ---
const ongoingColumns: ColumnDef<GameInfo>[] = [
    {
        accessorKey: "id",
        header: "Game ID",
        cell: ({ row }) => {
            const id = row.getValue("id") as string
            return <div className="font-mono text-xs text-muted-foreground">{id.split("-")[0]}...</div>
        },
    },
    {
        id: "format",
        header: "Format",
        cell: ({ row }) => {
            const team1 = row.original.team1.length
            const team2 = row.original.team2.length
            return <Badge variant="secondary">{team1} v {team2}</Badge>
        },
    },
    {
        id: "status",
        header: "Status",
        cell: () => (
            <Badge variant="outline" className="bg-blue-500/10 text-blue-500 border-blue-500/20">
                In Progress
            </Badge>
        ),
    },
    {
        accessorKey: "elapsed_seconds",
        header: "Match Duration",
        cell: ({ row }) => {
            const seconds = row.getValue("elapsed_seconds") as number
            return (
                <div className="flex items-center gap-2 font-mono">
                    <ClockIcon className="size-3 text-muted-foreground" />
                    {formatTime(seconds)}
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
    const [pagination, setPagination] = React.useState({ pageIndex: 0, pageSize: 10 })

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
export function WaitingGamesTable({ data }: { data: GameInfo[] }) {
    return <BaseTable columns={waitingColumns} data={data} emptyMessage="No games currently waiting for confirmation." />
}

export function OngoingGamesTable({ data }: { data: GameInfo[] }) {
    return <BaseTable columns={ongoingColumns} data={data} emptyMessage="No games are currently in progress." />
}