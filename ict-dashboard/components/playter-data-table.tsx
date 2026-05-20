"use client"

import * as React from "react"
import {
    flexRender,
    getCoreRowModel,
    getFilteredRowModel,
    getPaginationRowModel,
    getSortedRowModel,
    useReactTable,
    type ColumnDef,
    type ColumnFiltersState,
    type SortingState,
} from "@tanstack/react-table"
import {
    ChevronLeftIcon,
    ChevronRightIcon,
    ChevronsLeftIcon,
    ChevronsRightIcon,
    SearchIcon,
} from "lucide-react"

import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import {
    Select,
    SelectContent,
    SelectGroup,
    SelectItem,
    SelectTrigger,
    SelectValue,
} from "@/components/ui/select"
import {
    Table,
    TableBody,
    TableCell,
    TableHead,
    TableHeader,
    TableRow,
} from "@/components/ui/table"
import { PlayerInfo } from "@/hooks/useDashboard"

const columns: ColumnDef<PlayerInfo>[] = [
    {
        accessorKey: "name",
        header: "Name",
        cell: ({ row }) => <div className="font-semibold">{row.getValue("name")}</div>,
    },
    {
        accessorKey: "status",
        header: "Status",
        cell: ({ row }) => {

            const status = row.getValue("status") as PlayerInfo["status"]

            const getStatusColor = (s: PlayerInfo["status"]) => {
                if (typeof s === "string") {
                    switch (s) {
                        case "Idle": return "bg-green-500/10 text-green-500 border-green-500/20"
                        case "InQueue": return "bg-blue-500/10 text-blue-500 border-blue-500/20"
                        default: return "bg-gray-500/10 text-gray-500 border-gray-500/20"
                    }
                } else if ("NeedConfirmation" in s) {
                    return "bg-yellow-500/10 text-yellow-500 border-yellow-500/20"
                } else if ("InGame" in s) {
                    return "bg-red-500/10 text-red-500 border-red-500/20"
                } else {
                    return "bg-gray-500/10 text-gray-500 border-gray-500/20"
                }
            }

            const statusText = typeof status === "string" ? status : ("NeedConfirmation" in status ? "Need Confirmation" : ("InGame" in status ? "In Game" : "Unknown"))

            return (
                <Badge variant="outline" className={getStatusColor(status)}>
                    {statusText}
                </Badge>
            )
        },
    },
    {
        accessorKey: "mmr",
        header: "MMR",
        cell: ({ row }) => <div className="font-mono">{row.getValue("mmr")}</div>,
    },
    {
        accessorKey: "rank",
        header: "Rank",
        cell: ({ row }) => <div>{row.getValue("rank")}</div>,
    },
    {
        accessorKey: "div",
        header: "Division",
        cell: ({ row }) => <div>{row.getValue("div")}</div>,
    },
    {
        accessorKey: "points",
        header: "Points",
        cell: ({ row }) => <div className="font-mono">{row.getValue("points")}</div>,
    },
    {
        accessorKey: "skills",
        header: "Skills",
        cell: ({ row }) => {
            const skills = row.getValue("skills") as string[]
            if (!skills || skills.length === 0) return <span className="text-muted-foreground text-xs">None</span>

            return (
                <div className="flex flex-wrap gap-1 w-48">
                    {skills.map((skill) => (
                        <Badge key={skill} variant="secondary" className="px-1 text-[10px] uppercase font-mono">
                            {skill}
                        </Badge>
                    ))}
                </div>
            )
        },
    },
]

export function PlayerDataTable({ data }: { data: PlayerInfo[] }) {
    const [sorting, setSorting] = React.useState<SortingState>([])
    const [columnFilters, setColumnFilters] = React.useState<ColumnFiltersState>([])
    const [pagination, setPagination] = React.useState({
        pageIndex: 0,
        pageSize: 10,
    })

    const table = useReactTable({
        data,
        columns,
        state: {
            sorting,
            columnFilters,
            pagination,
        },
        onSortingChange: setSorting,
        onColumnFiltersChange: setColumnFilters,
        onPaginationChange: setPagination,
        getCoreRowModel: getCoreRowModel(),
        getFilteredRowModel: getFilteredRowModel(),
        getPaginationRowModel: getPaginationRowModel(),
        getSortedRowModel: getSortedRowModel(),
    })

    return (
        <div className="w-full flex flex-col gap-4">
            {/* Table Toolbar */}
            <div className="flex items-center justify-between">
                <div className="relative w-72">
                    <SearchIcon className="absolute left-2.5 top-2.5 h-4 w-4 text-muted-foreground" />
                    <Input
                        placeholder="Search players by name..."
                        value={(table.getColumn("name")?.getFilterValue() as string) ?? ""}
                        onChange={(event) =>
                            table.getColumn("name")?.setFilterValue(event.target.value)
                        }
                        className="pl-8"
                    />
                </div>
            </div>

            {/* Table Container */}
            <div className="rounded-md border overflow-hidden">
                <Table>
                    <TableHeader className="bg-muted/50">
                        {table.getHeaderGroups().map((headerGroup) => (
                            <TableRow key={headerGroup.id}>
                                {headerGroup.headers.map((header) => {
                                    return (
                                        <TableHead key={header.id}>
                                            {header.isPlaceholder
                                                ? null
                                                : flexRender(
                                                    header.column.columnDef.header,
                                                    header.getContext()
                                                )}
                                        </TableHead>
                                    )
                                })}
                            </TableRow>
                        ))}
                    </TableHeader>
                    <TableBody>
                        {table.getRowModel().rows?.length ? (
                            table.getRowModel().rows.map((row) => (
                                <TableRow key={row.id}>
                                    {row.getVisibleCells().map((cell) => (
                                        <TableCell key={cell.id}>
                                            {flexRender(
                                                cell.column.columnDef.cell,
                                                cell.getContext()
                                            )}
                                        </TableCell>
                                    ))}
                                </TableRow>
                            ))
                        ) : (
                            <TableRow>
                                <TableCell
                                    colSpan={columns.length}
                                    className="h-24 text-center text-muted-foreground"
                                >
                                    No players found.
                                </TableCell>
                            </TableRow>
                        )}
                    </TableBody>
                </Table>
            </div>

            {/* Pagination Controls */}
            <div className="flex items-center justify-between">
                <div className="text-sm text-muted-foreground">
                    Showing {table.getFilteredRowModel().rows.length} players
                </div>
                <div className="flex items-center gap-4 lg:w-fit">
                    <div className="flex items-center gap-2">
                        <span className="text-sm font-medium">Rows per page</span>
                        <Select
                            value={`${table.getState().pagination.pageSize}`}
                            onValueChange={(value) => {
                                table.setPageSize(Number(value))
                            }}
                        >
                            <SelectTrigger size="sm" className="w-[70px]">
                                <SelectValue placeholder={table.getState().pagination.pageSize} />
                            </SelectTrigger>
                            <SelectContent side="top">
                                {[10, 20, 30, 40, 50].map((pageSize) => (
                                    <SelectItem key={pageSize} value={`${pageSize}`}>
                                        {pageSize}
                                    </SelectItem>
                                ))}
                            </SelectContent>
                        </Select>
                    </div>
                    <div className="flex w-[100px] items-center justify-center text-sm font-medium">
                        Page {table.getState().pagination.pageIndex + 1} of{" "}
                        {table.getPageCount()}
                    </div>
                    <div className="flex items-center gap-2">
                        <Button
                            variant="outline"
                            className="h-8 w-8 p-0"
                            onClick={() => table.setPageIndex(0)}
                            disabled={!table.getCanPreviousPage()}
                        >
                            <ChevronsLeftIcon className="h-4 w-4" />
                        </Button>
                        <Button
                            variant="outline"
                            className="h-8 w-8 p-0"
                            onClick={() => table.previousPage()}
                            disabled={!table.getCanPreviousPage()}
                        >
                            <ChevronLeftIcon className="h-4 w-4" />
                        </Button>
                        <Button
                            variant="outline"
                            className="h-8 w-8 p-0"
                            onClick={() => table.nextPage()}
                            disabled={!table.getCanNextPage()}
                        >
                            <ChevronRightIcon className="h-4 w-4" />
                        </Button>
                        <Button
                            variant="outline"
                            className="h-8 w-8 p-0"
                            onClick={() => table.setPageIndex(table.getPageCount() - 1)}
                            disabled={!table.getCanNextPage()}
                        >
                            <ChevronsRightIcon className="h-4 w-4" />
                        </Button>
                    </div>
                </div>
            </div>
        </div>
    )
}