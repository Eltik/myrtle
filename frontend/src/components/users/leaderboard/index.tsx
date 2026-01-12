"use client";

import { ArrowDown, ArrowUp, ChevronLeft, ChevronRight, Trophy } from "lucide-react";
import Link from "next/link";
import { useRouter } from "next/router";
import { useCallback, useState } from "react";
import { Avatar, AvatarFallback, AvatarImage } from "~/components/ui/shadcn/avatar";
import { Badge } from "~/components/ui/shadcn/badge";
import { Button } from "~/components/ui/shadcn/button";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "~/components/ui/shadcn/select";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "~/components/ui/shadcn/table";
import { cn } from "~/lib/utils";
import type { LeaderboardEntry, LeaderboardResponse, SortBy } from "~/types/api";
import { generatePaginationItems } from "../shared/pagination";
import { getAvatarUrl, SERVERS, SORT_OPTIONS } from "./impl/constants";
import { GradeBadge } from "./impl/grade-badge";
import { LeaderboardRowDialog } from "./impl/leaderboard-row-dialog";
import { RankBadge } from "./impl/rank-badge";
import { ScoreBreakdownTooltip } from "./impl/score-breakdown-tooltip";

interface LeaderboardPageProps {
    initialData: LeaderboardResponse;
}

export function LeaderboardPage({ initialData }: LeaderboardPageProps) {
    const router = useRouter();
    const [data, setData] = useState(initialData);
    const [isLoading, setIsLoading] = useState(false);
    const [selectedEntry, setSelectedEntry] = useState<LeaderboardEntry | null>(null);

    // Get current filter values from URL or defaults
    const currentSortBy = (router.query.sort_by as SortBy) || "total_score";
    const currentOrder = (router.query.order as "asc" | "desc") || "desc";
    const currentServer = (router.query.server as string) || "all";
    const currentOffset = Number(router.query.offset) || 0;
    const limit = data.pagination.limit;

    const updateFilters = useCallback(
        async (newParams: Record<string, string | undefined>) => {
            setIsLoading(true);
            const query = { ...router.query, ...newParams };

            // Remove undefined/null values and "all" server filter
            for (const key of Object.keys(query)) {
                if (query[key] === undefined || query[key] === null || (key === "server" && query[key] === "all")) {
                    delete query[key];
                }
            }

            await router.push({ pathname: router.pathname, query }, undefined, { shallow: true });

            try {
                const params = new URLSearchParams();
                for (const [key, value] of Object.entries(query)) {
                    if (value) params.set(key, String(value));
                }

                const response = await fetch(`/api/leaderboard?${params.toString()}`);
                if (response.ok) {
                    const newData = (await response.json()) as LeaderboardResponse;
                    setData(newData);
                }
            } catch (error) {
                console.error("Failed to fetch leaderboard:", error);
            } finally {
                setIsLoading(false);
            }
        },
        [router],
    );

    const handleSortChange = (sortBy: string) => {
        void updateFilters({ sort_by: sortBy, offset: "0" });
    };

    const handleOrderToggle = () => {
        void updateFilters({ order: currentOrder === "desc" ? "asc" : "desc", offset: "0" });
    };

    const handleServerChange = (server: string) => {
        void updateFilters({ server: server === "all" ? undefined : server, offset: "0" });
    };

    const handlePageChange = (newOffset: number) => {
        void updateFilters({ offset: String(newOffset) });
        window.scrollTo({ top: 0, behavior: "smooth" });
    };

    const currentPage = Math.floor(currentOffset / limit) + 1;
    const totalPages = Math.ceil(data.pagination.total / limit);

    return (
        <div className="min-w-0 space-y-6">
            {/* Header */}
            <div className="space-y-2">
                <div className="flex items-center gap-3">
                    <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-primary/10">
                        <Trophy className="h-5 w-5 text-primary" />
                    </div>
                    <div>
                        <h1 className="font-bold text-3xl text-foreground md:text-4xl">Leaderboard</h1>
                        <p className="text-muted-foreground">Top players ranked by collection and progress</p>
                    </div>
                </div>
            </div>

            {/* Filters */}
            <div className="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
                <div className="flex flex-wrap items-center gap-2">
                    {/* Server Filter */}
                    <Select onValueChange={handleServerChange} value={currentServer}>
                        <SelectTrigger className="w-32">
                            <SelectValue placeholder="Server" />
                        </SelectTrigger>
                        <SelectContent>
                            {SERVERS.map((server) => (
                                <SelectItem key={server.value} value={server.value}>
                                    {server.label}
                                </SelectItem>
                            ))}
                        </SelectContent>
                    </Select>

                    {/* Sort By */}
                    <div className="flex h-9 items-center gap-1 rounded-md border border-input bg-transparent px-1">
                        <Select onValueChange={handleSortChange} value={currentSortBy}>
                            <SelectTrigger className="h-7 w-32 border-0 bg-transparent px-2 text-sm shadow-none focus:ring-0">
                                <SelectValue placeholder="Sort by" />
                            </SelectTrigger>
                            <SelectContent>
                                {SORT_OPTIONS.map((option) => (
                                    <SelectItem key={option.value} value={option.value}>
                                        {option.label}
                                    </SelectItem>
                                ))}
                            </SelectContent>
                        </Select>
                        <button className="flex h-7 w-7 shrink-0 items-center justify-center rounded text-muted-foreground transition-colors hover:bg-secondary hover:text-foreground" onClick={handleOrderToggle} type="button">
                            {currentOrder === "desc" ? <ArrowDown className="h-4 w-4" /> : <ArrowUp className="h-4 w-4" />}
                        </button>
                    </div>
                </div>

                {/* Results info */}
                <div className="text-muted-foreground text-sm">
                    {data.pagination.total > 0 ? (
                        <span>
                            Showing {currentOffset + 1}-{Math.min(currentOffset + limit, data.pagination.total)} of {data.pagination.total.toLocaleString()} players
                        </span>
                    ) : (
                        <span>No players found</span>
                    )}
                </div>
            </div>

            {/* Table */}
            <div className={cn("rounded-lg border bg-card/50 transition-opacity", isLoading && "opacity-60")}>
                <Table>
                    <TableHeader>
                        <TableRow className="hover:bg-transparent">
                            <TableHead className="w-20 py-3 text-center">Rank</TableHead>
                            <TableHead className="py-3">Player</TableHead>
                            <TableHead className="hidden py-3 text-center sm:table-cell">Grade</TableHead>
                            <TableHead className="hidden py-3 text-right md:table-cell">
                                <ScoreBreakdownTooltip>
                                    <span className="cursor-help">Score</span>
                                </ScoreBreakdownTooltip>
                            </TableHead>
                            <TableHead className="hidden py-3 text-center lg:table-cell">Server</TableHead>
                            <TableHead className="hidden py-3 text-center xl:table-cell">Level</TableHead>
                        </TableRow>
                    </TableHeader>
                    <TableBody>
                        {data.entries.length > 0 ? (
                            data.entries.map((entry) => (
                                <TableRow className="cursor-pointer transition-colors hover:bg-muted/50" key={`${entry.uid}-${entry.server}`} onClick={() => setSelectedEntry(entry)}>
                                    <TableCell className="py-4 text-center">
                                        <RankBadge rank={entry.rank} />
                                    </TableCell>
                                    <TableCell className="py-4">
                                        <div className="flex items-center gap-4">
                                            <Avatar className="h-11 w-11 border border-border">
                                                <AvatarImage alt={entry.nickname} src={getAvatarUrl(entry.avatarId)} />
                                                <AvatarFallback className="text-sm">{entry.nickname.slice(0, 2).toUpperCase()}</AvatarFallback>
                                            </Avatar>
                                            <div className="min-w-0">
                                                <Link className="block truncate font-medium text-base hover:text-primary hover:underline" href={`/user/${entry.uid}`} onClick={(e) => e.stopPropagation()}>
                                                    {entry.nickname}
                                                </Link>
                                                <div className="flex items-center gap-2 text-muted-foreground text-sm sm:hidden">
                                                    <GradeBadge grade={entry.grade} size="sm" />
                                                    <span>{entry.totalScore.toLocaleString()}</span>
                                                </div>
                                            </div>
                                        </div>
                                    </TableCell>
                                    <TableCell className="hidden py-4 text-center sm:table-cell">
                                        <GradeBadge grade={entry.grade} />
                                    </TableCell>
                                    <TableCell className="hidden py-4 text-right font-mono text-base md:table-cell">{entry.totalScore.toLocaleString()}</TableCell>
                                    <TableCell className="hidden py-4 text-center lg:table-cell">
                                        <Badge className="uppercase" variant="secondary">
                                            {entry.server}
                                        </Badge>
                                    </TableCell>
                                    <TableCell className="hidden py-4 text-center xl:table-cell">
                                        <span className="text-muted-foreground">{entry.level}</span>
                                    </TableCell>
                                </TableRow>
                            ))
                        ) : (
                            <TableRow>
                                <TableCell className="h-32 text-center text-muted-foreground" colSpan={6}>
                                    No players found matching your criteria.
                                </TableCell>
                            </TableRow>
                        )}
                    </TableBody>
                </Table>
            </div>

            {/* Pagination */}
            {totalPages > 1 && (
                <div className="flex items-center justify-between">
                    <Button disabled={currentPage === 1 || isLoading} onClick={() => handlePageChange(Math.max(0, currentOffset - limit))} variant="outline">
                        <ChevronLeft className="mr-1 h-4 w-4" />
                        Previous
                    </Button>
                    <div className="flex items-center gap-1">
                        {generatePaginationItems(currentPage, totalPages).map((item) =>
                            item.type === "ellipsis" ? (
                                <span className="px-2 text-muted-foreground" key={`ellipsis-${item.position}`}>
                                    ...
                                </span>
                            ) : (
                                <Button className={cn("h-9 w-9", currentPage === item.value && "pointer-events-none")} disabled={isLoading} key={item.value} onClick={() => handlePageChange((item.value - 1) * limit)} variant={currentPage === item.value ? "default" : "ghost"}>
                                    {item.value}
                                </Button>
                            ),
                        )}
                    </div>
                    <Button disabled={currentPage === totalPages || isLoading} onClick={() => handlePageChange(currentOffset + limit)} variant="outline">
                        Next
                        <ChevronRight className="ml-1 h-4 w-4" />
                    </Button>
                </div>
            )}

            {/* Detail Dialog */}
            <LeaderboardRowDialog entry={selectedEntry} onClose={() => setSelectedEntry(null)} />
        </div>
    );
}
