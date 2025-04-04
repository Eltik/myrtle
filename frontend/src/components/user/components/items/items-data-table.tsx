import { flexRender, type ColumnFiltersState, getCoreRowModel, getFilteredRowModel, getPaginationRowModel, getSortedRowModel, useReactTable, type SortingState } from "@tanstack/react-table";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "~/components/ui/table";
import type { DataTableProps } from "~/types/impl/frontend/impl/users";
import { useState } from "react";
import ItemDialogueCard from "./item-dialogue-card";
import { type Item } from "~/types/impl/api/static/material";
import { Dialog, DialogContent, DialogTrigger } from "~/components/ui/dialog";
import { Input } from "~/components/ui/input";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "~/components/ui/select";
import { Pagination, PaginationContent, PaginationEllipsis, PaginationItem, PaginationLink } from "~/components/ui/pagination";
import { ChevronLeftIcon, ChevronRightIcon } from "lucide-react";
import { Button } from "~/components/ui/button";

export function ItemsDataTable<TData, TValue>({ columns, data }: DataTableProps<TData, TValue>) {
    const [sorting, setSorting] = useState<SortingState>([]);
    const [columnFilters, setColumnFilters] = useState<ColumnFiltersState>([]);
    const [pageSize, setPageSize] = useState(10);
    const [pageIndex, setPageIndex] = useState(0);

    const table = useReactTable({
        data,
        columns,
        getCoreRowModel: getCoreRowModel(),
        onSortingChange: setSorting,
        getSortedRowModel: getSortedRowModel(),
        onColumnFiltersChange: setColumnFilters,
        getFilteredRowModel: getFilteredRowModel(),
        getPaginationRowModel: getPaginationRowModel(),
        state: {
            sorting,
            columnFilters,
            pagination: {
                pageSize,
                pageIndex,
            },
        },
        onPaginationChange: (updater) => {
            if (typeof updater === "function") {
                const newState = updater({ pageIndex, pageSize });
                setPageIndex(newState.pageIndex);
                setPageSize(newState.pageSize);
            } else {
                setPageIndex(updater.pageIndex);
                setPageSize(updater.pageSize);
            }
        },
    });

    const currentPage = table.getState().pagination.pageIndex;
    const totalPages = table.getPageCount();

    const getPageRange = () => {
        const maxButtons = 5;
        const halfButtons = Math.floor(maxButtons / 2);

        let startPage = Math.max(0, currentPage - halfButtons);
        const endPage = Math.min(totalPages - 1, startPage + maxButtons - 1);

        if (endPage - startPage + 1 < maxButtons) {
            startPage = Math.max(0, endPage - maxButtons + 1);
        }

        return Array.from({ length: endPage - startPage + 1 }, (_, i) => startPage + i);
    };

    const pageRange = getPageRange();

    const handlePageSizeChange = (value: string) => {
        if (value === "all") {
            setPageSize(data.length);
        } else {
            setPageSize(Number(value));
        }
        setPageIndex(0); // Reset to first page when changing page size
    };

    return (
        <>
            <div className="flex items-center justify-between py-4">
                <Input className="max-w-sm" placeholder="Filter items..." value={(table.getColumn("name")?.getFilterValue() as string) ?? ""} onChange={(event) => table.getColumn("name")?.setFilterValue(event.target.value)} />
                <div className="flex items-center space-x-2">
                    <span className="text-sm text-muted-foreground">Rows per page:</span>
                    <Select value={pageSize === data.length ? "all" : pageSize.toString()} onValueChange={handlePageSizeChange}>
                        <SelectTrigger className="h-8 w-[70px]">
                            <SelectValue placeholder={pageSize} />
                        </SelectTrigger>
                        <SelectContent side="top">
                            <SelectItem value="5">5</SelectItem>
                            <SelectItem value="10">10</SelectItem>
                            <SelectItem value="20">20</SelectItem>
                            <SelectItem value="50">50</SelectItem>
                            <SelectItem value="all">All</SelectItem>
                        </SelectContent>
                    </Select>
                </div>
            </div>
            <div className="mx-auto w-full rounded-md border">
                <Table className="mx-auto">
                    <TableHeader>
                        {table.getHeaderGroups().map((headerGroup) => (
                            <TableRow key={headerGroup.id}>
                                {headerGroup.headers.map((header) => {
                                    return <TableHead key={header.id}>{header.isPlaceholder ? null : flexRender(header.column.columnDef.header, header.getContext())}</TableHead>;
                                })}
                            </TableRow>
                        ))}
                    </TableHeader>
                    <TableBody>
                        {table.getRowModel().rows?.length ? (
                            table.getRowModel().rows.map((row) => (
                                <Dialog key={row.id}>
                                    <DialogTrigger asChild>
                                        <TableRow data-state={row.getIsSelected() && "selected"} className="cursor-pointer">
                                            {row.getVisibleCells().map((cell) => (
                                                <TableCell key={cell.id}>{flexRender(cell.column.columnDef.cell, cell.getContext())}</TableCell>
                                            ))}
                                        </TableRow>
                                    </DialogTrigger>
                                    <DialogContent>
                                        <ItemDialogueCard item={row.original as Item} />
                                    </DialogContent>
                                </Dialog>
                            ))
                        ) : (
                            <TableRow>
                                <TableCell colSpan={columns.length} className="h-24 text-center">
                                    No results.
                                </TableCell>
                            </TableRow>
                        )}
                    </TableBody>
                </Table>
            </div>
            {pageSize !== data.length && (
                <div className="flex items-center justify-between space-x-2 py-4">
                    <Pagination>
                        <PaginationContent>
                            <PaginationItem>
                                <Button variant="outline" size="icon" onClick={() => table.previousPage()} disabled={!table.getCanPreviousPage()}>
                                    <span className="sr-only">Go to previous page</span>
                                    <ChevronLeftIcon className="h-4 w-4" />
                                </Button>
                            </PaginationItem>
                            {(pageRange[0] ?? 0) > 0 && (
                                <>
                                    <PaginationItem>
                                        <PaginationLink className="cursor-pointer" onClick={() => table.setPageIndex(0)}>
                                            1
                                        </PaginationLink>
                                    </PaginationItem>
                                    {(pageRange[0] ?? 0) > 1 && <PaginationEllipsis />}
                                </>
                            )}
                            {pageRange.map((page) => (
                                <PaginationItem key={page}>
                                    <PaginationLink className="cursor-pointer" isActive={page === currentPage} onClick={() => table.setPageIndex(page)}>
                                        {page + 1}
                                    </PaginationLink>
                                </PaginationItem>
                            ))}
                            {(pageRange[pageRange.length - 1] ?? 0) < totalPages - 1 && (
                                <>
                                    {(pageRange[pageRange.length - 1] ?? 0) < totalPages - 2 && <PaginationEllipsis />}
                                    <PaginationItem>
                                        <PaginationLink className="cursor-pointer" onClick={() => table.setPageIndex(totalPages - 1)}>
                                            {totalPages}
                                        </PaginationLink>
                                    </PaginationItem>
                                </>
                            )}
                            <PaginationItem>
                                <Button variant="outline" size="icon" onClick={() => table.nextPage()} disabled={!table.getCanNextPage()}>
                                    <span className="sr-only">Go to next page</span>
                                    <ChevronRightIcon className="h-4 w-4" />
                                </Button>
                            </PaginationItem>
                        </PaginationContent>
                    </Pagination>
                </div>
            )}
        </>
    );
}
