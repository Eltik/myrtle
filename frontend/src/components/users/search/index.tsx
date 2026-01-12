"use client";

import { ArrowDown, ArrowUp, ChevronDown, ChevronLeft, ChevronRight, Filter, Search, Users, X } from "lucide-react";
import Link from "next/link";
import { useRouter } from "next/router";
import { memo, useCallback, useEffect, useMemo, useRef, useState } from "react";
import { InView } from "~/components/ui/motion-primitives/in-view";
import { Avatar, AvatarFallback, AvatarImage } from "~/components/ui/shadcn/avatar";
import { Badge } from "~/components/ui/shadcn/badge";
import { Button } from "~/components/ui/shadcn/button";
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "~/components/ui/shadcn/collapsible";
import { Input } from "~/components/ui/shadcn/input";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "~/components/ui/shadcn/select";
import { clearSearchAbortController, fetchSearchResultsCached, getSearchAbortController } from "~/lib/search-utils";
import { cn } from "~/lib/utils";
import type { SearchQuery, SearchResponse, SearchResultEntry, SearchServer } from "~/types/api";
import { getAvatarUrl } from "../leaderboard/impl/constants";

// Constants
const SERVERS: { value: SearchServer | "all"; label: string }[] = [
    { value: "all", label: "All Servers" },
    { value: "en", label: "Global (EN)" },
    { value: "jp", label: "Japan" },
    { value: "cn", label: "China" },
    { value: "kr", label: "Korea" },
    { value: "tw", label: "Taiwan" },
    { value: "bili", label: "Bilibili" },
];

const SERVER_COLORS: Record<string, string> = {
    en: "bg-blue-500/15 text-blue-400 border-blue-500/30",
    jp: "bg-pink-500/15 text-pink-400 border-pink-500/30",
    kr: "bg-purple-500/15 text-purple-400 border-purple-500/30",
    cn: "bg-red-500/15 text-red-400 border-red-500/30",
    bili: "bg-cyan-500/15 text-cyan-400 border-cyan-500/30",
    tw: "bg-green-500/15 text-green-400 border-green-500/30",
};

const SORT_OPTIONS = [
    { value: "nickname", label: "Name" },
    { value: "level", label: "Level" },
    { value: "updated_at", label: "Last Updated" },
    { value: "register_ts", label: "Join Date" },
] as const;

interface SearchPageProps {
    initialData: SearchResponse;
}

export function SearchPage({ initialData }: SearchPageProps) {
    const router = useRouter();
    const [data, setData] = useState(initialData);
    const [isLoading, setIsLoading] = useState(false);
    const [filtersOpen, setFiltersOpen] = useState(false);

    // Search state
    const [searchInput, setSearchInput] = useState((router.query.nickname as string) || "");
    const [debouncedSearch, setDebouncedSearch] = useState(searchInput);

    // Level filter state (controlled inputs)
    const [minLevel, setMinLevel] = useState((router.query.level as string)?.split(",")[0] || "");
    const [maxLevel, setMaxLevel] = useState((router.query.level as string)?.split(",")[1] || "");

    // Filter state (managed locally to avoid stale router.query issues)
    const [currentServer, setCurrentServer] = useState((router.query.server as string) || "all");
    const [currentSortBy, setCurrentSortBy] = useState((router.query.sortBy as string) || "nickname");
    const [currentOrder, setCurrentOrder] = useState<"asc" | "desc">((router.query.order as "asc" | "desc") || "asc");
    const [currentOffset, setCurrentOffset] = useState(Number(router.query.offset) || 0);

    // Track if this is the initial mount to avoid unnecessary fetches
    const isInitialMount = useRef(true);

    const limit = data.pagination.limit;

    // Sync level inputs with URL when URL changes externally
    // biome-ignore lint/correctness/useExhaustiveDependencies: Only sync when URL changes, not when local state changes
    useEffect(() => {
        const urlMinLevel = (router.query.level as string)?.split(",")[0] || "";
        const urlMaxLevel = (router.query.level as string)?.split(",")[1] || "";
        if (urlMinLevel !== minLevel) setMinLevel(urlMinLevel);
        if (urlMaxLevel !== maxLevel) setMaxLevel(urlMaxLevel);
    }, [router.query.level]);

    // Debounce search input
    useEffect(() => {
        const timer = setTimeout(() => {
            setDebouncedSearch(searchInput);
        }, 300);
        return () => clearTimeout(timer);
    }, [searchInput]);

    // Trigger search when debounced value changes
    // biome-ignore lint/correctness/useExhaustiveDependencies: Only trigger on debouncedSearch change, updateFilters is stable via useCallback
    useEffect(() => {
        // Skip on initial mount - SSR already provides data
        if (isInitialMount.current) {
            isInitialMount.current = false;
            return;
        }

        // Read current nickname from URL directly (not router.query which may be stale)
        const currentUrl = new URL(window.location.href);
        const currentNickname = currentUrl.searchParams.get("nickname") || "";

        if (debouncedSearch !== currentNickname) {
            void updateFilters({ nickname: debouncedSearch || undefined, offset: "0" });
        }
    }, [debouncedSearch]);

    const updateFilters = useCallback(async (newParams: Record<string, string | undefined>) => {
        setIsLoading(true);

        // Read current params from URL directly (not router.query which may be stale)
        const currentUrl = new URL(window.location.href);
        const currentParams: Record<string, string> = {};
        currentUrl.searchParams.forEach((value, key) => {
            currentParams[key] = value;
        });

        // Merge with new params
        const query: Record<string, string | undefined> = { ...currentParams, ...newParams };

        // Remove undefined/null values and "all" server filter
        for (const key of Object.keys(query)) {
            if (query[key] === undefined || query[key] === null || query[key] === "" || (key === "server" && query[key] === "all")) {
                delete query[key];
            }
        }

        // Build SearchQuery from the cleaned query params
        const searchQuery: SearchQuery = {};
        for (const [key, value] of Object.entries(query)) {
            if (value) {
                if (key === "limit" || key === "offset") {
                    (searchQuery as Record<string, unknown>)[key] = Number(value);
                } else {
                    (searchQuery as Record<string, unknown>)[key] = String(value);
                }
            }
        }

        try {
            // Get abort controller for this request (cancels any pending request)
            const controller = getSearchAbortController();

            const newData = await fetchSearchResultsCached(searchQuery, {
                signal: controller.signal,
            });

            setData(newData);
            clearSearchAbortController();

            // Update URL after successful fetch using replace to avoid re-render cascade
            // Use window.history directly to avoid triggering Next.js router re-renders
            const url = new URL(window.location.href);
            url.search = "";
            for (const [key, value] of Object.entries(query)) {
                if (value) url.searchParams.set(key, String(value));
            }
            window.history.replaceState({ ...window.history.state, as: url.pathname + url.search, url: url.pathname + url.search }, "", url.pathname + url.search);
        } catch (error) {
            // Don't log abort errors - they're expected when requests are canceled
            if (error instanceof Error && error.name === "AbortError") {
                return;
            }
            console.error("Failed to fetch search results:", error);
        } finally {
            setIsLoading(false);
        }
    }, []);

    const handleServerChange = (server: string) => {
        setCurrentServer(server);
        setCurrentOffset(0);
        void updateFilters({ server: server === "all" ? undefined : server, offset: "0" });
    };

    const handleSortChange = (sortBy: string) => {
        setCurrentSortBy(sortBy);
        setCurrentOffset(0);
        void updateFilters({ sortBy, offset: "0" });
    };

    const handleOrderToggle = () => {
        const newOrder = currentOrder === "desc" ? "asc" : "desc";
        setCurrentOrder(newOrder);
        setCurrentOffset(0);
        void updateFilters({ order: newOrder, offset: "0" });
    };

    const handleLevelFilter = (min: string, max: string) => {
        let levelParam: string | undefined;
        if (min && max) {
            levelParam = `${min},${max}`;
        } else if (min) {
            levelParam = min;
        } else if (max) {
            levelParam = `,${max}`;
        }
        setCurrentOffset(0);
        void updateFilters({ level: levelParam, offset: "0" });
    };

    const handlePageChange = (newOffset: number) => {
        setCurrentOffset(newOffset);
        void updateFilters({ offset: String(newOffset) });
        window.scrollTo({ top: 0, behavior: "smooth" });
    };

    const clearFilters = () => {
        setSearchInput("");
        setMinLevel("");
        setMaxLevel("");
        setCurrentServer("all");
        setCurrentSortBy("nickname");
        setCurrentOrder("asc");
        setCurrentOffset(0);
        void updateFilters({
            nickname: undefined,
            server: undefined,
            level: undefined,
            sortBy: "nickname",
            order: "asc",
            offset: "0",
        });
    };

    const hasActiveFilters = useMemo(() => {
        return searchInput !== "" || currentServer !== "all" || minLevel !== "" || maxLevel !== "";
    }, [searchInput, currentServer, minLevel, maxLevel]);

    const currentPage = Math.floor(currentOffset / limit) + 1;
    const totalPages = Math.ceil(data.pagination.total / limit);

    return (
        <div className="min-w-0 space-y-6">
            {/* Header */}
            <InView
                once
                transition={{ duration: 0.4, ease: "easeOut" }}
                variants={{
                    hidden: { opacity: 0, y: -10 },
                    visible: { opacity: 1, y: 0 },
                }}
            >
                <div className="space-y-2">
                    <div className="flex items-center gap-3">
                        <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-primary/10">
                            <Users className="h-5 w-5 text-primary" />
                        </div>
                        <div>
                            <h1 className="font-bold text-3xl text-foreground md:text-4xl">Player Search</h1>
                            <p className="text-muted-foreground">Find Arknights players by name, server, or level</p>
                        </div>
                    </div>
                </div>
            </InView>

            {/* Search Bar */}
            <InView
                once
                transition={{ duration: 0.4, delay: 0.1, ease: "easeOut" }}
                variants={{
                    hidden: { opacity: 0, y: 10 },
                    visible: { opacity: 1, y: 0 },
                }}
            >
                <div className="relative">
                    <Search className="absolute top-1/2 left-4 h-5 w-5 -translate-y-1/2 text-muted-foreground z-5" />
                    <Input className="h-12 rounded-xl border-border/50 bg-card/80 pr-12 pl-12 text-base shadow-sm backdrop-blur-sm transition-all focus:border-primary/50 focus:bg-card focus:shadow-md" onChange={(e) => setSearchInput(e.target.value)} placeholder="Search by nickname..." value={searchInput} />
                    {searchInput && (
                        <button className="absolute top-1/2 right-4 -translate-y-1/2 rounded-full p-1 text-muted-foreground transition-colors hover:bg-secondary hover:text-foreground" onClick={() => setSearchInput("")} type="button">
                            <X className="h-4 w-4" />
                        </button>
                    )}
                </div>
            </InView>

            {/* Filters Section */}
            <InView
                once
                transition={{ duration: 0.4, delay: 0.15, ease: "easeOut" }}
                variants={{
                    hidden: { opacity: 0, y: 10 },
                    visible: { opacity: 1, y: 0 },
                }}
            >
                <div className="space-y-4">
                    {/* Quick Filters Row */}
                    <div className="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
                        <div className="flex flex-wrap items-center gap-2">
                            {/* Server Filter */}
                            <Select onValueChange={handleServerChange} value={currentServer}>
                                <SelectTrigger className="h-9 w-32 border-border/50 bg-card/80">
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

                            {/* Sort Options */}
                            <div className="flex h-9 items-center gap-1 rounded-md border border-border/50 bg-card/80 px-1">
                                <Select onValueChange={handleSortChange} value={currentSortBy}>
                                    <SelectTrigger className="h-7 w-28 border-0 bg-transparent px-2 text-sm shadow-none focus:ring-0">
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
                                    {currentOrder === "asc" ? <ArrowUp className="h-3.5 w-3.5" /> : <ArrowDown className="h-3.5 w-3.5" />}
                                </button>
                            </div>

                            {/* Advanced Filters Toggle */}
                            <Collapsible onOpenChange={setFiltersOpen} open={filtersOpen}>
                                <CollapsibleTrigger asChild>
                                    <Button className="h-9 gap-1.5 bg-transparent" size="sm" variant="outline">
                                        <Filter className="h-3.5 w-3.5" />
                                        <span className="hidden sm:inline">Filters</span>
                                        <ChevronDown className={cn("h-3.5 w-3.5 transition-transform", filtersOpen && "rotate-180")} />
                                        {hasActiveFilters && <span className="size-1.5 rounded-full bg-primary" />}
                                    </Button>
                                </CollapsibleTrigger>
                            </Collapsible>

                            {hasActiveFilters && (
                                <Button className="h-9" onClick={clearFilters} size="sm" variant="ghost">
                                    Clear all
                                </Button>
                            )}
                        </div>

                        {/* Results count */}
                        <div className="text-muted-foreground text-sm">
                            {data.pagination.total > 0 ? (
                                <span>
                                    {currentOffset + 1}-{Math.min(currentOffset + limit, data.pagination.total)} of {data.pagination.total.toLocaleString()} players
                                </span>
                            ) : (
                                <span>No players found</span>
                            )}
                        </div>
                    </div>

                    {/* Advanced Filters Panel */}
                    <Collapsible onOpenChange={setFiltersOpen} open={filtersOpen}>
                        <CollapsibleContent>
                            <div className="rounded-lg border border-border/50 bg-card/50 p-4">
                                <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
                                    {/* Level Range */}
                                    <div className="space-y-2">
                                        <label className="font-medium text-muted-foreground text-sm" htmlFor="level-min">
                                            Level Range
                                        </label>
                                        <div className="flex items-center gap-2">
                                            <Input
                                                className="h-9"
                                                id="level-min"
                                                max={150}
                                                min={1}
                                                onBlur={() => handleLevelFilter(minLevel, maxLevel)}
                                                onChange={(e) => setMinLevel(e.target.value)}
                                                onKeyDown={(e) => {
                                                    if (e.key === "Enter") {
                                                        handleLevelFilter(minLevel, maxLevel);
                                                    }
                                                }}
                                                placeholder="Min"
                                                type="number"
                                                value={minLevel}
                                            />
                                            <span className="text-muted-foreground">-</span>
                                            <Input
                                                className="h-9"
                                                max={150}
                                                min={1}
                                                onBlur={() => handleLevelFilter(minLevel, maxLevel)}
                                                onChange={(e) => setMaxLevel(e.target.value)}
                                                onKeyDown={(e) => {
                                                    if (e.key === "Enter") {
                                                        handleLevelFilter(minLevel, maxLevel);
                                                    }
                                                }}
                                                placeholder="Max"
                                                type="number"
                                                value={maxLevel}
                                            />
                                        </div>
                                    </div>
                                </div>
                            </div>
                        </CollapsibleContent>
                    </Collapsible>
                </div>
            </InView>

            {/* Results Grid */}
            <div className={cn("transition-opacity", isLoading && "opacity-60")}>
                {data.results.length > 0 ? (
                    <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
                        {data.results.map((entry) => (
                            <UserCard entry={entry} key={`${entry.uid}-${entry.server}`} />
                        ))}
                    </div>
                ) : (
                    <div className="flex flex-col items-center justify-center rounded-xl border border-border/50 border-dashed bg-card/30 py-16 text-center">
                        <Users className="mb-4 h-12 w-12 text-muted-foreground/50" />
                        <h3 className="mb-1 font-medium text-lg">No players found</h3>
                        <p className="text-muted-foreground text-sm">Try adjusting your search or filters to find players</p>
                    </div>
                )}
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
        </div>
    );
}

// User Card Component - memoized to prevent unnecessary re-renders
const UserCard = memo(function UserCard({ entry }: { entry: SearchResultEntry }) {
    const serverColor = SERVER_COLORS[entry.server] || "bg-zinc-500/15 text-zinc-400 border-zinc-500/30";

    return (
        <Link className="group block rounded-xl border border-border/50 bg-card/80 p-4 shadow-sm backdrop-blur-sm transition-all hover:border-border hover:bg-card hover:shadow-md" href={`/user/${entry.uid}`}>
            <div className="flex items-start gap-4">
                <Avatar className="h-14 w-14 border border-border/50 transition-transform group-hover:scale-105">
                    <AvatarImage alt={entry.nickname} src={getAvatarUrl(entry.avatarId) || "/placeholder.svg"} />
                    <AvatarFallback className="text-base">{entry.nickname.slice(0, 2).toUpperCase()}</AvatarFallback>
                </Avatar>
                <div className="min-w-0 flex-1">
                    <div className="flex items-center gap-2">
                        <h3 className="truncate font-semibold text-base transition-colors group-hover:text-primary">{entry.nickname}</h3>
                    </div>
                    <div className="mt-1 flex flex-wrap items-center gap-2">
                        <Badge className={cn("border text-xs", serverColor)} variant="outline">
                            {entry.server.toUpperCase()}
                        </Badge>
                        <span className="text-muted-foreground text-sm">Lv. {entry.level}</span>
                    </div>
                </div>
            </div>
            {/* Updated time */}
            <div className="mt-3 border-border/30 border-t pt-3">
                <p className="text-muted-foreground text-xs">Last updated {formatRelativeTime(entry.updatedAt)}</p>
            </div>
        </Link>
    );
});

// Helper function for relative time
function formatRelativeTime(dateString: string): string {
    const date = new Date(dateString);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));

    if (diffDays === 0) return "today";
    if (diffDays === 1) return "yesterday";
    if (diffDays < 7) return `${diffDays} days ago`;
    if (diffDays < 30) return `${Math.floor(diffDays / 7)} weeks ago`;
    if (diffDays < 365) return `${Math.floor(diffDays / 30)} months ago`;
    return `${Math.floor(diffDays / 365)} years ago`;
}

// Pagination helper
type PaginationItem = { type: "page"; value: number } | { type: "ellipsis"; position: "start" | "end" };

function generatePaginationItems(currentPage: number, totalPages: number): PaginationItem[] {
    const items: PaginationItem[] = [];

    if (totalPages <= 7) {
        for (let i = 1; i <= totalPages; i++) items.push({ type: "page", value: i });
        return items;
    }

    items.push({ type: "page", value: 1 });

    if (currentPage > 3) {
        items.push({ type: "ellipsis", position: "start" });
    }

    const start = Math.max(2, currentPage - 1);
    const end = Math.min(totalPages - 1, currentPage + 1);

    for (let i = start; i <= end; i++) {
        items.push({ type: "page", value: i });
    }

    if (currentPage < totalPages - 2) {
        items.push({ type: "ellipsis", position: "end" });
    }

    items.push({ type: "page", value: totalPages });

    return items;
}
