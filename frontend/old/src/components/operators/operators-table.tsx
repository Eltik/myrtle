import type { ColumnDef } from "@tanstack/react-table";
import { type ColumnFiltersState, flexRender, getCoreRowModel, getFilteredRowModel, getSortedRowModel, type SortingState, useReactTable } from "@tanstack/react-table";
import { ArrowUpDown } from "lucide-react";
import { useRouter } from "next/navigation";
import { useState } from "react";
import { Button } from "~/components/ui/button";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "~/components/ui/table";
import { formatProfession, formatSubProfession, rarityToNumber } from "~/helper";
import type { Operator } from "~/types/impl/api/static/operator";
import { OperatorRarity } from "~/types/impl/api/static/operator";

interface OperatorsTableProps {
    operators: Operator[];
    currentPage: number;
    pageSize: number;
}

export function OperatorsTable({ operators, currentPage, pageSize }: OperatorsTableProps) {
    const [sorting, setSorting] = useState<SortingState>([]);
    const [columnFilters, setColumnFilters] = useState<ColumnFiltersState>([]);
    const router = useRouter();

    const handleRowClick = (operatorId: string | undefined) => {
        if (operatorId) {
            router.push(`/operators?id=${operatorId}`);
        }
    };

    const columns: ColumnDef<Operator>[] = [
        {
            accessorKey: "name",
            header: ({ column }) => {
                return (
                    <Button onClick={() => column.toggleSorting(column.getIsSorted() === "asc")} variant="ghost">
                        Name
                        <ArrowUpDown className="ml-2 h-4 w-4" />
                    </Button>
                );
            },
        },
        {
            accessorKey: "rarity",
            header: ({ column }) => {
                return (
                    <Button onClick={() => column.toggleSorting(column.getIsSorted() === "asc")} variant="ghost">
                        Rarity
                        <ArrowUpDown className="ml-2 h-4 w-4" />
                    </Button>
                );
            },
            cell: ({ row }) => {
                const rarity = row.getValue("rarity");
                return (
                    <div className="flex items-center gap-2">
                        <span
                            className={`font-bold ${rarity === OperatorRarity.sixStar ? "text-[#f7a452]" : rarity === OperatorRarity.fiveStar ? "text-[#f7e79e]" : rarity === OperatorRarity.fourStar ? "text-[#bcabdb]" : rarity === OperatorRarity.threeStar ? "text-[#88c8e3]" : rarity === OperatorRarity.twoStar ? "text-[#7ef2a3]" : "text-white"}`}
                        >
                            {rarityToNumber(rarity as OperatorRarity)}★
                        </span>
                    </div>
                );
            },
        },
        {
            accessorKey: "profession",
            header: ({ column }) => {
                return (
                    <Button onClick={() => column.toggleSorting(column.getIsSorted() === "asc")} variant="ghost">
                        Class
                        <ArrowUpDown className="ml-2 h-4 w-4" />
                    </Button>
                );
            },
            cell: ({ row }) => {
                const profession = row.getValue("profession");
                return <span>{formatProfession(profession as string)}</span>;
            },
        },
        {
            accessorKey: "subProfessionId",
            header: ({ column }) => {
                return (
                    <Button onClick={() => column.toggleSorting(column.getIsSorted() === "asc")} variant="ghost">
                        Subclass
                        <ArrowUpDown className="ml-2 h-4 w-4" />
                    </Button>
                );
            },
            cell: ({ row }) => {
                const subProfessionId = row.getValue("subProfessionId");
                return <span>{formatSubProfession((subProfessionId as string).toLowerCase())}</span>;
            },
        },
        {
            accessorKey: "nationId",
            header: ({ column }) => {
                return (
                    <Button onClick={() => column.toggleSorting(column.getIsSorted() === "asc")} variant="ghost">
                        Nation
                        <ArrowUpDown className="ml-2 h-4 w-4" />
                    </Button>
                );
            },
        },
        {
            accessorKey: "profile.basicInfo.race",
            header: ({ column }) => {
                return (
                    <Button onClick={() => column.toggleSorting(column.getIsSorted() === "asc")} variant="ghost">
                        Race
                        <ArrowUpDown className="ml-2 h-4 w-4" />
                    </Button>
                );
            },
        },
    ];

    const table = useReactTable({
        data: operators,
        columns,
        getCoreRowModel: getCoreRowModel(),
        onSortingChange: setSorting,
        getSortedRowModel: getSortedRowModel(),
        onColumnFiltersChange: setColumnFilters,
        getFilteredRowModel: getFilteredRowModel(),
        state: {
            sorting,
            columnFilters,
        },
    });

    // Calculate pagination
    const startIndex = (currentPage - 1) * pageSize;
    const endIndex = startIndex + pageSize;
    const paginatedData = table.getRowModel().rows.slice(startIndex, endIndex);

    return (
        <div className="mx-auto w-full rounded-md border">
            <Table>
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
                    {paginatedData.length ? (
                        paginatedData.map((row) => (
                            <TableRow className="cursor-pointer hover:bg-muted/50" key={row.id} onClick={() => handleRowClick(row.original.id)}>
                                {row.getVisibleCells().map((cell) => (
                                    <TableCell key={cell.id}>{flexRender(cell.column.columnDef.cell, cell.getContext())}</TableCell>
                                ))}
                            </TableRow>
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
    );
}
