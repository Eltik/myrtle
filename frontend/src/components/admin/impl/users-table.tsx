"use client";

import { ArrowUpDown, ChevronLeft, ChevronRight, ChevronsLeft, ChevronsRight, RefreshCw, Search, SlidersHorizontal } from "lucide-react";
import { useMemo, useState } from "react";
import type { RecentUser } from "~/types/frontend/impl/admin";
import { Avatar, AvatarFallback } from "../../ui/shadcn/avatar";
import { Badge } from "../../ui/shadcn/badge";
import { Button } from "../../ui/shadcn/button";
import { DropdownMenu, DropdownMenuCheckboxItem, DropdownMenuContent, DropdownMenuGroup, DropdownMenuSeparator, DropdownMenuTrigger } from "../../ui/shadcn/dropdown-menu";
import { Input } from "../../ui/shadcn/input";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "../../ui/shadcn/select";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "../../ui/shadcn/table";

interface UsersTableProps {
    users: RecentUser[];
    loading?: boolean;
    onRefresh?: () => void;
}

type SortField = "nickname" | "uid" | "level" | "server" | "role" | "createdAt";
type SortOrder = "asc" | "desc";

const ROLE_COLORS: Record<string, string> = {
    super_admin: "bg-red-500/10 text-red-500 border-red-500/20",
    tier_list_admin: "bg-orange-500/10 text-orange-500 border-orange-500/20",
    tier_list_editor: "bg-blue-500/10 text-blue-500 border-blue-500/20",
    user: "bg-zinc-500/10 text-zinc-400 border-zinc-500/20",
};

const SERVER_COLORS: Record<string, string> = {
    en: "bg-blue-500/10 text-blue-400 border-blue-500/20",
    jp: "bg-pink-500/10 text-pink-400 border-pink-500/20",
    kr: "bg-purple-500/10 text-purple-400 border-purple-500/20",
    cn: "bg-red-500/10 text-red-400 border-red-500/20",
    bili: "bg-cyan-500/10 text-cyan-400 border-cyan-500/20",
    tw: "bg-green-500/10 text-green-400 border-green-500/20",
};

function RoleBadge({ role }: { role: string }) {
    const colorClass = ROLE_COLORS[role] ?? ROLE_COLORS.user;
    const displayName = role.replace(/_/g, " ").replace(/\b\w/g, (c) => c.toUpperCase());

    return (
        <Badge className={`${colorClass} border font-medium`} variant="outline">
            {displayName}
        </Badge>
    );
}

function ServerBadge({ server }: { server: string }) {
    const colorClass = SERVER_COLORS[server] ?? "bg-zinc-500/10 text-zinc-400 border-zinc-500/20";

    return (
        <Badge className={`${colorClass} border font-mono text-xs`} variant="outline">
            {server.toUpperCase()}
        </Badge>
    );
}

export function UsersTable({ users, loading = false, onRefresh }: UsersTableProps) {
    const [searchQuery, setSearchQuery] = useState("");
    const [roleFilter, setRoleFilter] = useState<string>("all");
    const [serverFilter, setServerFilter] = useState<string>("all");
    const [sortField, setSortField] = useState<SortField>("createdAt");
    const [sortOrder, setSortOrder] = useState<SortOrder>("desc");
    const [currentPage, setCurrentPage] = useState(1);
    const [itemsPerPage, setItemsPerPage] = useState(10);

    const filteredAndSortedUsers = useMemo(() => {
        const result = users.filter((user) => {
            const matchesSearch = searchQuery === "" || user.nickname.toLowerCase().includes(searchQuery.toLowerCase()) || user.uid.toLowerCase().includes(searchQuery.toLowerCase());

            const matchesRole = roleFilter === "all" || user.role === roleFilter;
            const matchesServer = serverFilter === "all" || user.server === serverFilter;

            return matchesSearch && matchesRole && matchesServer;
        });

        result.sort((a, b) => {
            let comparison = 0;
            switch (sortField) {
                case "nickname":
                    comparison = a.nickname.localeCompare(b.nickname);
                    break;
                case "uid":
                    comparison = a.uid.localeCompare(b.uid);
                    break;
                case "level":
                    comparison = a.level - b.level;
                    break;
                case "server":
                    comparison = a.server.localeCompare(b.server);
                    break;
                case "role":
                    comparison = a.role.localeCompare(b.role);
                    break;
                case "createdAt":
                    comparison = new Date(a.createdAt).getTime() - new Date(b.createdAt).getTime();
                    break;
            }
            return sortOrder === "asc" ? comparison : -comparison;
        });

        return result;
    }, [users, searchQuery, roleFilter, serverFilter, sortField, sortOrder]);

    const totalPages = Math.ceil(filteredAndSortedUsers.length / itemsPerPage);
    const paginatedUsers = useMemo(() => {
        const startIndex = (currentPage - 1) * itemsPerPage;
        return filteredAndSortedUsers.slice(startIndex, startIndex + itemsPerPage);
    }, [filteredAndSortedUsers, currentPage, itemsPerPage]);

    const toggleSort = (field: SortField) => {
        if (sortField === field) {
            setSortOrder(sortOrder === "asc" ? "desc" : "asc");
        } else {
            setSortField(field);
            setSortOrder("asc");
        }
    };

    const hasActiveFilters = searchQuery !== "" || roleFilter !== "all" || serverFilter !== "all";

    const clearFilters = () => {
        setSearchQuery("");
        setRoleFilter("all");
        setServerFilter("all");
    };

    const handlePageChange = (page: number) => {
        setCurrentPage(Math.max(1, Math.min(page, totalPages)));
    };

    // Get unique servers from users
    const availableServers = useMemo(() => {
        return [...new Set(users.map((u) => u.server))].sort();
    }, [users]);

    // Get unique roles from users
    const availableRoles = useMemo(() => {
        return [...new Set(users.map((u) => u.role))].sort();
    }, [users]);

    return (
        <div className="overflow-hidden rounded-xl border bg-card text-card-foreground">
            {/* Header */}
            <div className="flex flex-col gap-3 border-b px-4 py-3.5 sm:flex-row sm:items-center sm:justify-between">
                <div className="flex items-center gap-3">
                    <h3 className="font-medium text-base">User Management</h3>
                    <div className="hidden h-5 w-px bg-border sm:block" />
                    <div className="hidden items-center gap-2 sm:flex">
                        {/* Search */}
                        <div className="relative">
                            <Search className="absolute top-1/2 left-2.5 h-3.5 w-3.5 -translate-y-1/2 text-muted-foreground" />
                            <Input className="h-8 w-50 border-border/50 bg-muted/50 pl-8 text-sm" onChange={(e) => setSearchQuery(e.target.value)} placeholder="Search users..." value={searchQuery} />
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
                                {/* Role Filter */}
                                <DropdownMenuGroup>
                                    <div className="px-2 py-1.5">
                                        <p className="mb-1.5 font-medium text-muted-foreground text-xs">Role</p>
                                        <div className="space-y-1">
                                            <DropdownMenuCheckboxItem checked={roleFilter === "all"} onCheckedChange={() => setRoleFilter("all")}>
                                                All Roles
                                            </DropdownMenuCheckboxItem>
                                            {availableRoles.map((role) => (
                                                <DropdownMenuCheckboxItem checked={roleFilter === role} key={role} onCheckedChange={() => setRoleFilter(role)}>
                                                    {role.replace(/_/g, " ")}
                                                </DropdownMenuCheckboxItem>
                                            ))}
                                        </div>
                                    </div>
                                </DropdownMenuGroup>
                                <DropdownMenuSeparator />

                                {/* Server Filter */}
                                <DropdownMenuGroup>
                                    <div className="px-2 py-1.5">
                                        <p className="mb-1.5 font-medium text-muted-foreground text-xs">Server</p>
                                        <div className="space-y-1">
                                            <DropdownMenuCheckboxItem checked={serverFilter === "all"} onCheckedChange={() => setServerFilter("all")}>
                                                All Servers
                                            </DropdownMenuCheckboxItem>
                                            {availableServers.map((server) => (
                                                <DropdownMenuCheckboxItem checked={serverFilter === server} key={server} onCheckedChange={() => setServerFilter(server)}>
                                                    {server.toUpperCase()}
                                                </DropdownMenuCheckboxItem>
                                            ))}
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
                    <span className="text-muted-foreground text-sm">{filteredAndSortedUsers.length} users</span>
                    {onRefresh && (
                        <Button className="h-8 gap-1.5" disabled={loading} onClick={onRefresh} size="sm" variant="outline">
                            <RefreshCw className={`h-3.5 w-3.5 ${loading ? "animate-spin" : ""}`} />
                            <span className="hidden sm:inline">Refresh</span>
                        </Button>
                    )}
                </div>
            </div>

            {/* Mobile Controls */}
            <div className="flex flex-wrap items-center gap-2 border-b px-4 py-3 sm:hidden">
                <div className="relative flex-1">
                    <Search className="absolute top-1/2 left-2.5 h-3.5 w-3.5 -translate-y-1/2 text-muted-foreground" />
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
                                <p className="mb-1.5 font-medium text-muted-foreground text-xs">Role</p>
                                <div className="space-y-1">
                                    <DropdownMenuCheckboxItem checked={roleFilter === "all"} onCheckedChange={() => setRoleFilter("all")}>
                                        All Roles
                                    </DropdownMenuCheckboxItem>
                                    {availableRoles.map((role) => (
                                        <DropdownMenuCheckboxItem checked={roleFilter === role} key={role} onCheckedChange={() => setRoleFilter(role)}>
                                            {role.replace(/_/g, " ")}
                                        </DropdownMenuCheckboxItem>
                                    ))}
                                </div>
                            </div>
                        </DropdownMenuGroup>
                        <DropdownMenuSeparator />
                        <DropdownMenuGroup>
                            <div className="px-2 py-1.5">
                                <p className="mb-1.5 font-medium text-muted-foreground text-xs">Server</p>
                                <div className="space-y-1">
                                    <DropdownMenuCheckboxItem checked={serverFilter === "all"} onCheckedChange={() => setServerFilter("all")}>
                                        All Servers
                                    </DropdownMenuCheckboxItem>
                                    {availableServers.map((server) => (
                                        <DropdownMenuCheckboxItem checked={serverFilter === server} key={server} onCheckedChange={() => setServerFilter(server)}>
                                            {server.toUpperCase()}
                                        </DropdownMenuCheckboxItem>
                                    ))}
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
                            <TableHead className="w-50">
                                <button className="flex items-center gap-1.5 text-muted-foreground hover:text-foreground" onClick={() => toggleSort("nickname")} type="button">
                                    <span>User</span>
                                    <ArrowUpDown className="h-3 w-3" />
                                </button>
                            </TableHead>
                            <TableHead className="w-30">
                                <button className="flex items-center gap-1.5 text-muted-foreground hover:text-foreground" onClick={() => toggleSort("uid")} type="button">
                                    <span>UID</span>
                                    <ArrowUpDown className="h-3 w-3" />
                                </button>
                            </TableHead>
                            <TableHead className="w-20">
                                <button className="flex items-center gap-1.5 text-muted-foreground hover:text-foreground" onClick={() => toggleSort("level")} type="button">
                                    <span>Level</span>
                                    <ArrowUpDown className="h-3 w-3" />
                                </button>
                            </TableHead>
                            <TableHead className="w-20">
                                <button className="flex items-center gap-1.5 text-muted-foreground hover:text-foreground" onClick={() => toggleSort("server")} type="button">
                                    <span>Server</span>
                                    <ArrowUpDown className="h-3 w-3" />
                                </button>
                            </TableHead>
                            <TableHead className="w-35">
                                <button className="flex items-center gap-1.5 text-muted-foreground hover:text-foreground" onClick={() => toggleSort("role")} type="button">
                                    <span>Role</span>
                                    <ArrowUpDown className="h-3 w-3" />
                                </button>
                            </TableHead>
                            <TableHead className="w-35">
                                <button className="flex items-center gap-1.5 text-muted-foreground hover:text-foreground" onClick={() => toggleSort("createdAt")} type="button">
                                    <span>Created</span>
                                    <ArrowUpDown className="h-3 w-3" />
                                </button>
                            </TableHead>
                        </TableRow>
                    </TableHeader>
                    <TableBody>
                        {paginatedUsers.length === 0 ? (
                            <TableRow>
                                <TableCell className="h-24 text-center text-muted-foreground" colSpan={6}>
                                    No users found.
                                </TableCell>
                            </TableRow>
                        ) : (
                            paginatedUsers.map((user) => (
                                <TableRow className="border-border/50" key={user.id}>
                                    <TableCell>
                                        <div className="flex items-center gap-2.5">
                                            <Avatar className="size-7">
                                                <AvatarFallback className="text-xs">{user.nickname[0]}</AvatarFallback>
                                            </Avatar>
                                            <span className="font-medium text-sm">{user.nickname}</span>
                                        </div>
                                    </TableCell>
                                    <TableCell>
                                        <span className="font-mono text-muted-foreground text-sm">{user.uid}</span>
                                    </TableCell>
                                    <TableCell>
                                        <span className="text-sm">{user.level}</span>
                                    </TableCell>
                                    <TableCell>
                                        <ServerBadge server={user.server} />
                                    </TableCell>
                                    <TableCell>
                                        <RoleBadge role={user.role} />
                                    </TableCell>
                                    <TableCell>
                                        <span className="text-muted-foreground text-sm">{new Date(user.createdAt).toLocaleDateString()}</span>
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
                            Showing {(currentPage - 1) * itemsPerPage + 1} to {Math.min(currentPage * itemsPerPage, filteredAndSortedUsers.length)} of {filteredAndSortedUsers.length}
                        </span>
                        <div className="hidden h-4 w-px bg-border sm:block" />
                        <div className="flex items-center gap-2">
                            <span className="hidden sm:inline">Show</span>
                            <Select onValueChange={(v) => setItemsPerPage(Number(v))} value={itemsPerPage.toString()}>
                                <SelectTrigger className="h-8 w-17.5 bg-muted/50">
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
