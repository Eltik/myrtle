import { flexRender, type ColumnFiltersState, getCoreRowModel, getFilteredRowModel, getPaginationRowModel, getSortedRowModel, useReactTable, type SortingState } from "@tanstack/react-table";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "~/components/ui/table";
import type { DataTableProps } from "~/types/impl/frontend/impl/users";
import { Button } from "../../ui/button";
import { Input } from "../../ui/input";
import { useEffect, useState } from "react";
import { Dialog, DialogContent, DialogTrigger } from "../../ui/dialog";
import CharacterDialogueCard from "./character-dialogue-card";
import type { CharacterData } from "~/types/impl/api";
import { Pagination, PaginationContent, PaginationItem, PaginationLink, PaginationNext, PaginationPrevious } from "~/components/ui/pagination";

const useWindowSize = () => {
    const [windowSize, setWindowSize] = useState({
        width: typeof window !== 'undefined' ? window.innerWidth : 0,
        height: typeof window !== 'undefined' ? window.innerHeight : 0,
    });

    useEffect(() => {
        const handleResize = () => {
            setWindowSize({
                width: window.innerWidth,
                height: window.innerHeight,
            });
        };

        window.addEventListener('resize', handleResize);
        handleResize();

        return () => window.removeEventListener('resize', handleResize);
    }, []);

    return windowSize;
}

export function CharactersDataTable<TData, TValue>({ columns, data }: DataTableProps<TData, TValue>) {
    const [sorting, setSorting] = useState<SortingState>([]);
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
        let endPage = Math.min(totalPages - 1, startPage + maxButtons - 1);
        
        if (endPage - startPage + 1 < maxButtons) {
            startPage = Math.max(0, endPage - maxButtons + 1);
        }
        
        return Array.from({length: endPage - startPage + 1}, (_, i) => startPage + i);
    };

    const pageRange = getPageRange();

    return (
        <>
            <div className="flex items-center py-4">
                <Input className="max-w-sm" placeholder="Search operators..." value={(table.getColumn("name")?.getFilterValue() as string) ?? ""} onChange={(event) => table.getColumn("name")?.setFilterValue(event.target.value)} />
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
                                        <CharacterDialogueCard data={row.original as CharacterData} />
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
            <div className="flex items-center justify-end space-x-2">
                <Button variant="outline" size="sm" onClick={() => table.previousPage()} disabled={!table.getCanPreviousPage()}>
                    Previous
                </Button>
                <Pagination>
                    <PaginationContent>
                        {table.getPageOptions().map((page) => (
                            <PaginationItem>
                                <Button key={page} variant="outline" size="sm" onClick={() => table.setPageIndex(page)}>
                                    {page + 1}
                                </Button>
                            </PaginationItem>
                        ))}
                    </PaginationContent>
                </Pagination>
                <Button variant="outline" size="sm" onClick={() => table.nextPage()} disabled={!table.getCanNextPage()}>
                    Next
                </Button>
            </div>
        </>
    );
}
