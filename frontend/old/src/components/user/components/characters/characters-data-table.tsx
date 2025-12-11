import { type ColumnFiltersState, flexRender, getCoreRowModel, getFilteredRowModel, getPaginationRowModel, getSortedRowModel, type SortingState, useReactTable } from "@tanstack/react-table";
import { ChevronLeftIcon, ChevronRightIcon } from "lucide-react";
import { useEffect, useState } from "react";
import { Button } from "~/components/ui/button";
import { Dialog, DialogContent, DialogTrigger } from "~/components/ui/dialog";
import { Input } from "~/components/ui/input";
import { Pagination, PaginationContent, PaginationEllipsis, PaginationItem, PaginationLink } from "~/components/ui/pagination";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "~/components/ui/table";
import type { CharacterData } from "~/types/impl/api";
import type { DataTableProps } from "~/types/impl/frontend/impl/users";
import CharacterDialogueCard from "./character-dialogue-card";

const useWindowSize = () => {
    const [windowSize, setWindowSize] = useState({
        width: typeof window !== "undefined" ? window.innerWidth : 0,
        height: typeof window !== "undefined" ? window.innerHeight : 0,
    });

    useEffect(() => {
        const handleResize = () => {
            setWindowSize({
                width: window.innerWidth,
                height: window.innerHeight,
            });
        };

        window.addEventListener("resize", handleResize);
        handleResize();

        return () => window.removeEventListener("resize", handleResize);
    }, []);

    return windowSize;
};

export function CharactersDataTable<TData, TValue>({ columns, data }: DataTableProps<TData, TValue>) {
    const [sorting, setSorting] = useState<SortingState>([{ id: "level", desc: true }]);
    const [columnFilters, setColumnFilters] = useState<ColumnFiltersState>([]);
    const { width } = useWindowSize();

    const table = useReactTable({
        data,
        columns,
        getCoreRowModel: getCoreRowModel(),
        getPaginationRowModel: getPaginationRowModel(),
        onSortingChange: setSorting,
        getSortedRowModel: getSortedRowModel(),
        onColumnFiltersChange: setColumnFilters,
        getFilteredRowModel: getFilteredRowModel(),
        state: {
            sorting,
            columnFilters,
        },
    });

    const currentPage = table.getState().pagination.pageIndex;
    const totalPages = table.getPageCount();

    const getPageRange = () => {
        const maxButtons = width < 640 ? 3 : width < 768 ? 5 : 7;
        const halfButtons = Math.floor(maxButtons / 2);

        let startPage = Math.max(0, currentPage - halfButtons);
        const endPage = Math.min(totalPages - 1, startPage + maxButtons - 1);

        if (endPage - startPage + 1 < maxButtons) {
            startPage = Math.max(0, endPage - maxButtons + 1);
        }

        return Array.from({ length: endPage - startPage + 1 }, (_, i) => startPage + i);
    };

    const pageRange = getPageRange();

    return (
        <>
            <div className="flex items-center py-4">
                <Input className="max-w-sm" onChange={(event) => table.getColumn("name")?.setFilterValue(event.target.value)} placeholder="Search operators..." value={(table.getColumn("name")?.getFilterValue() as string) ?? ""} />
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
                                        <TableRow className="cursor-pointer" data-state={row.getIsSelected() && "selected"}>
                                            {row.getVisibleCells().map((cell) => (
                                                <TableCell key={cell.id}>{flexRender(cell.column.columnDef.cell, cell.getContext())}</TableCell>
                                            ))}
                                        </TableRow>
                                    </DialogTrigger>
                                    <DialogContent>
                                        <CharacterDialogueCard data={row.original as CharacterData} />
                                    </DialogContent>
                                </Dialog>
                            ))
                        ) : (
                            <TableRow>
                                <TableCell className="h-24 text-center" colSpan={columns.length}>
                                    No results.
                                </TableCell>
                            </TableRow>
                        )}
                    </TableBody>
                </Table>
            </div>
            <div className="flex items-center justify-between space-x-2 py-4">
                <Pagination>
                    <PaginationContent>
                        <PaginationItem>
                            <Button disabled={!table.getCanPreviousPage()} onClick={() => table.previousPage()} size="icon" variant="outline">
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
                            <Button disabled={!table.getCanNextPage()} onClick={() => table.nextPage()} size="icon" variant="outline">
                                <span className="sr-only">Go to next page</span>
                                <ChevronRightIcon className="h-4 w-4" />
                            </Button>
                        </PaginationItem>
                    </PaginationContent>
                </Pagination>
            </div>
        </>
    );
}
