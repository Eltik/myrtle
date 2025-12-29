"use client";

import { ArrowUpDown, ChevronLeft, ChevronRight, ChevronsLeft, ChevronsRight, Edit, Eye, LayoutList, MoreHorizontal, Plus, RefreshCw, Search, SlidersHorizontal, Trash2 } from "lucide-react";
import { useMemo, useState } from "react";
import type { TierListSummary } from "~/types/frontend/admin";
import { Badge } from "../../ui/shadcn/badge";
import { Button } from "../../ui/shadcn/button";
import { DropdownMenu, DropdownMenuCheckboxItem, DropdownMenuContent, DropdownMenuGroup, DropdownMenuItem, DropdownMenuSeparator, DropdownMenuTrigger } from "../../ui/shadcn/dropdown-menu";
import { Input } from "../../ui/shadcn/input";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "../../ui/shadcn/select";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "../../ui/shadcn/table";

interface TierListsTableProps {
    tierLists: TierListSummary[];
    loading?: boolean;
    onRefresh?: () => void;
    onEdit?: (tierList: TierListSummary) => void;
    onDelete?: (tierList: TierListSummary) => void;
    onCreate?: () => void;
    onView?: (tierList: TierListSummary) => void;
}

type SortField = "name" | "operatorCount" | "tierCount" | "versionCount" | "createdAt" | "updatedAt";
type SortOrder = "asc" | "desc";

const STATUS_COLORS = {
    active: "bg-green-500/10 text-green-500 border-green-500/20",
    inactive: "bg-zinc-500/10 text-zinc-400 border-zinc-500/20",
};

function StatusBadge({ isActive }: { isActive: boolean }) {
    const colorClass = isActive ? STATUS_COLORS.active : STATUS_COLORS.inactive;
    return (
        <Badge className={`${colorClass} border font-medium`} variant="outline">
            {isActive ? "Active" : "Inactive"}
        </Badge>
    );
}

export function TierListsTable({ tierLists, loading = false, onRefresh, onEdit, onDelete, onCreate, onView }: TierListsTableProps) {
    const [searchQuery, setSearchQuery] = useState("");
    const [statusFilter, setStatusFilter] = useState<string>("all");
    const [sortField, setSortField] = useState<SortField>("updatedAt");
    const [sortOrder, setSortOrder] = useState<SortOrder>("desc");
    const [currentPage, setCurrentPage] = useState(1);
    const [itemsPerPage, setItemsPerPage] = useState(10);

    const filteredAndSortedTierLists = useMemo(() => {
        const result = tierLists.filter((tierList) => {
            const matchesSearch = searchQuery === "" || tierList.name.toLowerCase().includes(searchQuery.toLowerCase()) || tierList.slug.toLowerCase().includes(searchQuery.toLowerCase());

            const matchesStatus = statusFilter === "all" || (statusFilter === "active" ? tierList.isActive : !tierList.isActive);

            return matchesSearch && matchesStatus;
        });

        result.sort((a, b) => {
            let comparison = 0;
            switch (sortField) {
                case "name":
                    comparison = a.name.localeCompare(b.name);
                    break;
                case "operatorCount":
                    comparison = a.operatorCount - b.operatorCount;
                    break;
                case "tierCount":
                    comparison = a.tierCount - b.tierCount;
                    break;
                case "versionCount":
                    comparison = a.versionCount - b.versionCount;
                    break;
                case "createdAt":
                    comparison = new Date(a.createdAt).getTime() - new Date(b.createdAt).getTime();
                    break;
                case "updatedAt":
                    comparison = new Date(a.updatedAt).getTime() - new Date(b.updatedAt).getTime();
                    break;
            }
            return sortOrder === "asc" ? comparison : -comparison;
        });

        return result;
    }, [tierLists, searchQuery, statusFilter, sortField, sortOrder]);

    const totalPages = Math.ceil(filteredAndSortedTierLists.length / itemsPerPage);
    const paginatedTierLists = useMemo(() => {
        const startIndex = (currentPage - 1) * itemsPerPage;
        return filteredAndSortedTierLists.slice(startIndex, startIndex + itemsPerPage);
    }, [filteredAndSortedTierLists, currentPage, itemsPerPage]);

    const toggleSort = (field: SortField) => {
        if (sortField === field) {
            setSortOrder(sortOrder === "asc" ? "desc" : "asc");
        } else {
            setSortField(field);
            setSortOrder("asc");
        }
    };

    const hasActiveFilters = searchQuery !== "" || statusFilter !== "all";

    const clearFilters = () => {
        setSearchQuery("");
        setStatusFilter("all");
    };

    const handlePageChange = (page: number) => {
        setCurrentPage(Math.max(1, Math.min(page, totalPages)));
    };

    return (
        <div className="overflow-hidden rounded-xl border bg-card text-card-foreground">
            {/* Header */}
            <div className="flex flex-col gap-3 border-b px-4 py-3.5 sm:flex-row sm:items-center sm:justify-between">
                <div className="flex items-center gap-3">
                    <LayoutList className="size-5 text-primary" />
                    <h3 className="font-medium text-base">Tier List Management</h3>
                    <div className="hidden h-5 w-px bg-border sm:block" />
                    <div className="hidden items-center gap-2 sm:flex">
                        {/* Search */}
                        <div className="relative">
                            <Search className="-translate-y-1/2 absolute top-1/2 left-2.5 h-3.5 w-3.5 text-muted-foreground" />
                            <Input className="h-8 w-[200px] border-border/50 bg-muted/50 pl-8 text-sm" onChange={(e) => setSearchQuery(e.target.value)} placeholder="Search tier lists..." value={searchQuery} />
                        </div>

                        {/* Filter Dropdown */}
                        <DropdownMenu>
                            <DropdownMenuTrigger asChild>
                                <Button className="h-8 gap-1.5 border-border/50 bg-muted/50" size="sm" variant="outline">
                                    <SlidersHorizontal className="h-3.5 w-3.5" />
                                    <span>Filter</span>
                                    {hasActiveFilters && <span className="size-1.5 rounded-full bg-primary" />}
                                </Button>
                            </DropdownMenuTrigger>
                            <DropdownMenuContent align="start" className="w-48">
                                <DropdownMenuGroup>
                                    <div className="px-2 py-1.5">
                                        <p className="mb-1.5 font-medium text-muted-foreground text-xs">Status</p>
                                        <div className="space-y-1">
                                            <DropdownMenuCheckboxItem checked={statusFilter === "all"} onCheckedChange={() => setStatusFilter("all")}>
                                                All
                                            </DropdownMenuCheckboxItem>
                                            <DropdownMenuCheckboxItem checked={statusFilter === "active"} onCheckedChange={() => setStatusFilter("active")}>
                                                Active
                                            </DropdownMenuCheckboxItem>
                                            <DropdownMenuCheckboxItem checked={statusFilter === "inactive"} onCheckedChange={() => setStatusFilter("inactive")}>
                                                Inactive
                                            </DropdownMenuCheckboxItem>
                                        </div>
                                    </div>
                                </DropdownMenuGroup>

                                {hasActiveFilters && (
                                    <>
                                        <DropdownMenuSeparator />
                                        <Button className="w-full justify-start" onClick={clearFilters} size="sm" variant="ghost">
                                            Clear all filters
                                        </Button>
                                    </>
                                )}
                            </DropdownMenuContent>
                        </DropdownMenu>
                    </div>
                </div>

                <div className="flex items-center gap-2">
                    <span className="text-muted-foreground text-sm">{filteredAndSortedTierLists.length} tier lists</span>
                    {onCreate && (
                        <Button className="h-8 gap-1.5" onClick={onCreate} size="sm" variant="default">
                            <Plus className="h-3.5 w-3.5" />
                            <span className="hidden sm:inline">Create</span>
                        </Button>
                    )}
                    {onRefresh && (
                        <Button className="h-8 gap-1.5" disabled={loading} onClick={onRefresh} size="sm" variant="outline">
                            <RefreshCw className={`h-3.5 w-3.5 ${loading ? "animate-spin" : ""}`} />
                        </Button>
                    )}
                </div>
            </div>

            {/* Mobile Controls */}
            <div className="flex flex-wrap items-center gap-2 border-b px-4 py-3 sm:hidden">
                <div className="relative flex-1">
                    <Search className="-translate-y-1/2 absolute top-1/2 left-2.5 h-3.5 w-3.5 text-muted-foreground" />
                    <Input className="h-8 w-full border-border/50 bg-muted/50 pl-8 text-sm" onChange={(e) => setSearchQuery(e.target.value)} placeholder="Search..." value={searchQuery} />
                </div>
                <DropdownMenu>
                    <DropdownMenuTrigger asChild>
                        <Button className="h-8 gap-1.5 border-border/50 bg-muted/50" size="sm" variant="outline">
                            <SlidersHorizontal className="h-3.5 w-3.5" />
                            {hasActiveFilters && <span className="size-1.5 rounded-full bg-primary" />}
                        </Button>
                    </DropdownMenuTrigger>
                    <DropdownMenuContent align="end" className="w-48">
                        <DropdownMenuGroup>
                            <div className="px-2 py-1.5">
                                <p className="mb-1.5 font-medium text-muted-foreground text-xs">Status</p>
                                <div className="space-y-1">
                                    <DropdownMenuCheckboxItem checked={statusFilter === "all"} onCheckedChange={() => setStatusFilter("all")}>
                                        All
                                    </DropdownMenuCheckboxItem>
                                    <DropdownMenuCheckboxItem checked={statusFilter === "active"} onCheckedChange={() => setStatusFilter("active")}>
                                        Active
                                    </DropdownMenuCheckboxItem>
                                    <DropdownMenuCheckboxItem checked={statusFilter === "inactive"} onCheckedChange={() => setStatusFilter("inactive")}>
                                        Inactive
                                    </DropdownMenuCheckboxItem>
                                </div>
                            </div>
                        </DropdownMenuGroup>
                        {hasActiveFilters && (
                            <>
                                <DropdownMenuSeparator />
                                <Button className="w-full justify-start" onClick={clearFilters} size="sm" variant="ghost">
                                    Clear filters
                                </Button>
                            </>
                        )}
                    </DropdownMenuContent>
                </DropdownMenu>
            </div>

            {/* Table */}
            <div className="relative overflow-x-auto">
                {loading && (
                    <div className="absolute inset-0 z-10 flex items-center justify-center bg-background/50 backdrop-blur-sm">
                        <div className="h-6 w-6 animate-spin rounded-full border-2 border-primary border-t-transparent" />
                    </div>
                )}
                <Table>
                    <TableHeader>
                        <TableRow className="bg-muted/30 hover:bg-muted/30">
                            <TableHead className="w-[200px]">
                                <button className="flex items-center gap-1.5 text-muted-foreground hover:text-foreground" onClick={() => toggleSort("name")} type="button">
                                    <span>Name</span>
                                    <ArrowUpDown className="h-3 w-3" />
                                </button>
                            </TableHead>
                            <TableHead className="w-[100px]">Status</TableHead>
                            <TableHead className="w-20">
                                <button className="flex items-center gap-1.5 text-muted-foreground hover:text-foreground" onClick={() => toggleSort("tierCount")} type="button">
                                    <span>Tiers</span>
                                    <ArrowUpDown className="h-3 w-3" />
                                </button>
                            </TableHead>
                            <TableHead className="w-[100px]">
                                <button className="flex items-center gap-1.5 text-muted-foreground hover:text-foreground" onClick={() => toggleSort("operatorCount")} type="button">
                                    <span>Operators</span>
                                    <ArrowUpDown className="h-3 w-3" />
                                </button>
                            </TableHead>
                            <TableHead className="w-[100px]">
                                <button className="flex items-center gap-1.5 text-muted-foreground hover:text-foreground" onClick={() => toggleSort("versionCount")} type="button">
                                    <span>Versions</span>
                                    <ArrowUpDown className="h-3 w-3" />
                                </button>
                            </TableHead>
                            <TableHead className="w-[120px]">
                                <button className="flex items-center gap-1.5 text-muted-foreground hover:text-foreground" onClick={() => toggleSort("updatedAt")} type="button">
                                    <span>Updated</span>
                                    <ArrowUpDown className="h-3 w-3" />
                                </button>
                            </TableHead>
                            <TableHead className="w-[60px]">Actions</TableHead>
                        </TableRow>
                    </TableHeader>
                    <TableBody>
                        {paginatedTierLists.length === 0 ? (
                            <TableRow>
                                <TableCell className="h-24 text-center text-muted-foreground" colSpan={7}>
                                    No tier lists found.
                                </TableCell>
                            </TableRow>
                        ) : (
                            paginatedTierLists.map((tierList) => (
                                <TableRow className="border-border/50" key={tierList.id}>
                                    <TableCell>
                                        <div className="flex flex-col">
                                            <span className="font-medium text-sm">{tierList.name}</span>
                                            <span className="font-mono text-muted-foreground text-xs">{tierList.slug}</span>
                                        </div>
                                    </TableCell>
                                    <TableCell>
                                        <StatusBadge isActive={tierList.isActive} />
                                    </TableCell>
                                    <TableCell>
                                        <span className="font-mono text-sm">{tierList.tierCount}</span>
                                    </TableCell>
                                    <TableCell>
                                        <span className="font-mono text-sm">{tierList.operatorCount}</span>
                                    </TableCell>
                                    <TableCell>
                                        <span className="font-mono text-sm">{tierList.versionCount}</span>
                                    </TableCell>
                                    <TableCell>
                                        <span className="text-muted-foreground text-sm">{new Date(tierList.updatedAt).toLocaleDateString()}</span>
                                    </TableCell>
                                    <TableCell>
                                            <DropdownMenu>
                                                <DropdownMenuTrigger asChild>
                                                    <Button className="h-8 w-8" size="icon" variant="ghost">
                                                        <MoreHorizontal className="h-4 w-4" />
                                                    </Button>
                                                </DropdownMenuTrigger>
                                                <DropdownMenuContent align="end">
                                                    {onView && (
                                                        <DropdownMenuItem onClick={() => onView(tierList)}>
                                                            <Eye className="mr-2 h-4 w-4" />
                                                            View
                                                        </DropdownMenuItem>
                                                    )}
                                                    {onEdit && (
                                                        <DropdownMenuItem onClick={() => onEdit(tierList)}>
                                                            <Edit className="mr-2 h-4 w-4" />
                                                            Edit
                                                        </DropdownMenuItem>
                                                    )}
                                                    {onDelete && (
                                                        <>
                                                            <DropdownMenuSeparator />
                                                            <DropdownMenuItem className="text-destructive focus:text-destructive" onClick={() => onDelete(tierList)}>
                                                                <Trash2 className="mr-2 h-4 w-4" />
                                                                Delete
                                                            </DropdownMenuItem>
                                                        </>
                                                    )}
                                                </DropdownMenuContent>
                                            </DropdownMenu>
                                    </TableCell>
                                </TableRow>
                            ))
                        )}
                    </TableBody>
                </Table>
            </div>

            {/* Footer - Pagination */}
            {totalPages > 0 && (
                <div className="flex flex-col items-center justify-between gap-4 border-t px-4 py-3 sm:flex-row">
                    <div className="flex items-center gap-2 text-muted-foreground text-sm">
                        <span>
                            Showing {(currentPage - 1) * itemsPerPage + 1} to {Math.min(currentPage * itemsPerPage, filteredAndSortedTierLists.length)} of {filteredAndSortedTierLists.length}
                        </span>
                        <div className="hidden h-4 w-px bg-border sm:block" />
                        <div className="flex items-center gap-2">
                            <span className="hidden sm:inline">Show</span>
                            <Select onValueChange={(v) => setItemsPerPage(Number(v))} value={itemsPerPage.toString()}>
                                <SelectTrigger className="h-8 w-[70px] bg-muted/50">
                                    <SelectValue />
                                </SelectTrigger>
                                <SelectContent>
                                    <SelectItem value="5">5</SelectItem>
                                    <SelectItem value="10">10</SelectItem>
                                    <SelectItem value="20">20</SelectItem>
                                    <SelectItem value="50">50</SelectItem>
                                </SelectContent>
                            </Select>
                            <span className="hidden sm:inline">per page</span>
                        </div>
                    </div>

                    {/* Pagination Controls */}
                    <div className="flex items-center gap-1">
                        <Button className="size-8" disabled={currentPage === 1} onClick={() => handlePageChange(1)} size="icon" variant="outline">
                            <ChevronsLeft className="h-4 w-4" />
                        </Button>
                        <Button className="size-8" disabled={currentPage === 1} onClick={() => handlePageChange(currentPage - 1)} size="icon" variant="outline">
                            <ChevronLeft className="h-4 w-4" />
                        </Button>

                        <div className="mx-2 flex items-center gap-1">
                            {Array.from({ length: Math.min(5, totalPages) }, (_, i) => {
                                let pageNum: number;
                                if (totalPages <= 5) {
                                    pageNum = i + 1;
                                } else if (currentPage <= 3) {
                                    pageNum = i + 1;
                                } else if (currentPage >= totalPages - 2) {
                                    pageNum = totalPages - 4 + i;
                                } else {
                                    pageNum = currentPage - 2 + i;
                                }
                                return (
                                    <Button className="size-8" key={pageNum} onClick={() => handlePageChange(pageNum)} size="icon" variant={currentPage === pageNum ? "default" : "outline"}>
                                        {pageNum}
                                    </Button>
                                );
                            })}
                        </div>

                        <Button className="size-8" disabled={currentPage === totalPages} onClick={() => handlePageChange(currentPage + 1)} size="icon" variant="outline">
                            <ChevronRight className="h-4 w-4" />
                        </Button>
                        <Button className="size-8" disabled={currentPage === totalPages} onClick={() => handlePageChange(totalPages)} size="icon" variant="outline">
                            <ChevronsRight className="h-4 w-4" />
                        </Button>
                    </div>
                </div>
            )}
        </div>
    );
}
