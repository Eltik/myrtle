"use client";

import { ChevronDown, ChevronLeft, ChevronRight, Grid, List, RotateCcw, Search, SlidersHorizontal, Users } from "lucide-react";
import { AnimatePresence, motion } from "motion/react";
import { useRouter } from "next/router";
import { useCallback, useEffect, useRef, useState } from "react";
import { InView } from "~/components/ui/motion-primitives/in-view";
import { Button } from "~/components/ui/shadcn/button";
import { Collapsible, CollapsibleContent } from "~/components/ui/shadcn/collapsible";
import { Input } from "~/components/ui/shadcn/input";
import { Label } from "~/components/ui/shadcn/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "~/components/ui/shadcn/select";
import { clearSearchAbortController, fetchSearchResultsCached, getSearchAbortController } from "~/lib/search-utils";
import { cn } from "~/lib/utils";
import type { SearchGrade, SearchQuery, SearchResponse, SearchServer, SearchSortBy } from "~/types/api";
import { SERVERS } from "../leaderboard/impl/constants";
import { GRADES, SORT_OPTIONS } from "./impl/constants";
import { EmptyState } from "./impl/empty-state";
import { FilterPill } from "./impl/filter-pill";
import { generatePaginationItems } from "./impl/helpers";
import { SearchResultsGrid } from "./impl/search-results-grid";
import { SearchResultsList } from "./impl/search-results-list";

interface SearchPageContentProps {
    initialData: SearchResponse;
}

export function SearchPageContent({ initialData }: SearchPageContentProps) {
    const router = useRouter();
    const [data, setData] = useState(initialData);
    const [isLoading, setIsLoading] = useState(false);
    const [viewMode, setViewMode] = useState<"grid" | "list">("grid");
    const [filtersOpen, setFiltersOpen] = useState(false);
    const searchInputRef = useRef<HTMLInputElement>(null);

    const [nickname, setNickname] = useState((router.query.nickname as string) || "");
    const [uid, setUid] = useState((router.query.uid as string) || "");
    const [currentServer, setCurrentServer] = useState<SearchServer | "all">((router.query.server as SearchServer | "all") || "all");
    const [currentGrade, setCurrentGrade] = useState<SearchGrade | "all">((router.query.grade as SearchGrade | "all") || "all");
    const [currentSortBy, setCurrentSortBy] = useState<SearchSortBy>((router.query.sortBy as SearchSortBy) || "updated_at");
    const [currentOrder, setCurrentOrder] = useState<"asc" | "desc">((router.query.order as "asc" | "desc") || "desc");
    const [currentOffset, setCurrentOffset] = useState(Number(router.query.offset) || 0);
    const [levelMin, setLevelMin] = useState((router.query.levelMin as string) || "");
    const [levelMax, setLevelMax] = useState((router.query.levelMax as string) || "");

    const limit = data.pagination.limit;
    const totalPages = Math.ceil(data.pagination.total / limit);
    const currentPage = Math.floor(currentOffset / limit) + 1;

    const hasActiveFilters = !!(nickname || uid || currentServer !== "all" || currentGrade !== "all" || levelMin || levelMax);

    const updateSearch = useCallback(async (newParams: Record<string, string | undefined>, resetOffset = true) => {
        setIsLoading(true);

        const currentURL = new URL(window.location.href);
        const currentParams: Record<string, string | undefined> = {};
        currentURL.searchParams.forEach((value, key) => {
            currentParams[key] = value;
        });

        const query: Record<string, string | undefined> = {
            ...currentParams,
            ...newParams,
        };

        if (resetOffset) {
            query.offset = "0";
        }

        for (const key of Object.keys(query)) {
            if (query[key] === undefined || query[key] === null || query[key] === "" || (key === "server" && query[key] === "all") || (key === "grade" && query[key] === "all")) {
                delete query[key];
            }
        }

        try {
            const controller = getSearchAbortController();

            // Do NOT request fields=data -- it includes the entire game profile JSONB
            // which is very large. Hover cards lazy-load data on demand instead.
            const searchQuery: SearchQuery = {
                nickname: query.nickname,
                uid: query.uid,
                server: query.server as SearchServer | undefined,
                grade: query.grade as SearchGrade | undefined,
                sortBy: (query.sortBy as SearchSortBy) || "updated_at",
                order: (query.order as "asc" | "desc") || "desc",
                limit: 24,
                offset: Number(query.offset) || 0,
            };

            if (query.levelMin || query.levelMax) {
                searchQuery.level = `${query.levelMin || ""},${query.levelMax || ""}`;
            }

            const result = await fetchSearchResultsCached(searchQuery, { signal: controller.signal });
            setData(result);
            clearSearchAbortController();

            const newSearchParams = new URLSearchParams();
            for (const [key, value] of Object.entries(query)) {
                if (value !== undefined) {
                    newSearchParams.set(key, value);
                }
            }
            const newURL = `${window.location.pathname}${newSearchParams.toString() ? `?${newSearchParams.toString()}` : ""}`;
            window.history.replaceState({ ...window.history.state, as: newURL, url: newURL }, "", newURL);
        } catch (error) {
            if (error instanceof Error && error.name !== "AbortError") {
                console.error("Search failed:", error);
            }
        } finally {
            setIsLoading(false);
        }
    }, []);

    const searchTimeoutRef = useRef<NodeJS.Timeout | null>(null);

    const handleSearchInput = useCallback(
        (value: string, field: "nickname" | "uid") => {
            if (field === "nickname") {
                setNickname(value);
            } else {
                setUid(value);
            }

            if (searchTimeoutRef.current) {
                clearTimeout(searchTimeoutRef.current);
            }
            searchTimeoutRef.current = setTimeout(() => {
                void updateSearch({ [field]: value || undefined });
            }, 300);
        },
        [updateSearch],
    );

    const handleServerChange = (value: string) => {
        setCurrentServer(value as SearchServer | "all");
        void updateSearch({ server: value === "all" ? undefined : value });
    };

    const handleGradeChange = (value: string) => {
        setCurrentGrade(value as SearchGrade | "all");
        void updateSearch({ grade: value === "all" ? undefined : value });
    };

    const handleSortChange = (value: string) => {
        setCurrentSortBy(value as SearchSortBy);
        void updateSearch({ sortBy: value }, false);
    };

    const handleOrderToggle = () => {
        const newOrder = currentOrder === "desc" ? "asc" : "desc";
        setCurrentOrder(newOrder);
        void updateSearch({ order: newOrder }, false);
    };

    const handleLevelFilter = () => {
        void updateSearch({
            levelMin: levelMin || undefined,
            levelMax: levelMax || undefined,
        });
    };

    const handleResetFilters = () => {
        setNickname("");
        setUid("");
        setLevelMin("");
        setLevelMax("");
        setCurrentServer("all");
        setCurrentGrade("all");
        setCurrentSortBy("updated_at");
        setCurrentOrder("desc");
        setCurrentOffset(0);
        void updateSearch({
            nickname: undefined,
            uid: undefined,
            server: undefined,
            grade: undefined,
            levelMin: undefined,
            levelMax: undefined,
            sortBy: "updated_at",
            order: "desc",
        });
    };

    const handlePageChange = (newOffset: number) => {
        setCurrentOffset(newOffset);
        void updateSearch({ offset: String(newOffset) }, false);
        window.scrollTo({ top: 0, behavior: "smooth" });
    };

    useEffect(() => {
        const handleKeyDown = (e: KeyboardEvent) => {
            if ((e.metaKey || e.ctrlKey) && e.key === "k") {
                e.preventDefault();
                searchInputRef.current?.focus();
            }
        };
        window.addEventListener("keydown", handleKeyDown);
        return () => window.removeEventListener("keydown", handleKeyDown);
    }, []);

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
                            <p className="text-muted-foreground">Find Arknights players by name, UID, or filter by server and level</p>
                        </div>
                    </div>
                </div>
            </InView>

            {/* Main Search Bar */}
            <InView
                once
                transition={{ duration: 0.4, ease: "easeOut", delay: 0.05 }}
                variants={{
                    hidden: { opacity: 0, y: 10 },
                    visible: { opacity: 1, y: 0 },
                }}
            >
                <div className="relative">
                    <Search className="absolute top-1/2 left-3 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
                    <Input className="h-12 pr-20 pl-10 text-base" onChange={(e) => handleSearchInput(e.target.value, "nickname")} placeholder="Search by player name..." ref={searchInputRef} value={nickname} />
                    <kbd className="pointer-events-none absolute top-1/2 right-3 hidden -translate-y-1/2 select-none items-center gap-1 rounded border bg-muted px-1.5 py-0.5 font-medium font-mono text-muted-foreground text-xs sm:flex">
                        <span className="text-xs">⌘</span>K
                    </kbd>
                </div>
            </InView>

            {/* Filters Section */}
            <InView
                once
                transition={{ duration: 0.4, ease: "easeOut", delay: 0.1 }}
                variants={{
                    hidden: { opacity: 0, y: 10 },
                    visible: { opacity: 1, y: 0 },
                }}
            >
                <div className="space-y-4">
                    {/* Quick Filters Row */}
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

                        {/* Grade Filter */}
                        <Select onValueChange={handleGradeChange} value={currentGrade}>
                            <SelectTrigger className="w-32">
                                <SelectValue placeholder="Grade" />
                            </SelectTrigger>
                            <SelectContent>
                                {GRADES.map((grade) => (
                                    <SelectItem key={grade.value} value={grade.value}>
                                        {grade.label}
                                    </SelectItem>
                                ))}
                            </SelectContent>
                        </Select>

                        {/* Sort */}
                        <div className="flex h-9 items-center gap-1 rounded-md border border-input bg-transparent px-1">
                            <Select onValueChange={handleSortChange} value={currentSortBy}>
                                <SelectTrigger className="h-7 w-36 border-0 bg-transparent px-2 text-sm shadow-none focus:ring-0">
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
                                <motion.div animate={{ rotate: currentOrder === "asc" ? 180 : 0 }} transition={{ duration: 0.2 }}>
                                    <ChevronDown className="h-4 w-4" />
                                </motion.div>
                            </button>
                        </div>

                        {/* Advanced Filters Toggle */}
                        <Button className={cn("gap-2", filtersOpen && "bg-secondary")} onClick={() => setFiltersOpen(!filtersOpen)} size="sm" variant="outline">
                            <SlidersHorizontal className="h-4 w-4" />
                            <span className="hidden sm:inline">More Filters</span>
                            <motion.div animate={{ rotate: filtersOpen ? 180 : 0 }} transition={{ duration: 0.2 }}>
                                <ChevronDown className="h-3 w-3" />
                            </motion.div>
                        </Button>

                        {/* Reset Filters */}
                        <AnimatePresence>
                            {hasActiveFilters && (
                                <motion.div animate={{ opacity: 1, scale: 1 }} exit={{ opacity: 0, scale: 0.9 }} initial={{ opacity: 0, scale: 0.9 }}>
                                    <Button className="gap-1 text-muted-foreground" onClick={handleResetFilters} size="sm" variant="ghost">
                                        <RotateCcw className="h-3 w-3" />
                                        Reset
                                    </Button>
                                </motion.div>
                            )}
                        </AnimatePresence>

                        {/* Spacer */}
                        <div className="flex-1" />

                        {/* View Toggle */}
                        <div className="flex h-9 items-center rounded-md border border-input p-1">
                            <button className={cn("flex h-7 w-7 items-center justify-center rounded transition-colors", viewMode === "grid" ? "bg-secondary text-foreground" : "text-muted-foreground hover:text-foreground")} onClick={() => setViewMode("grid")} type="button">
                                <Grid className="h-4 w-4" />
                            </button>
                            <button className={cn("flex h-7 w-7 items-center justify-center rounded transition-colors", viewMode === "list" ? "bg-secondary text-foreground" : "text-muted-foreground hover:text-foreground")} onClick={() => setViewMode("list")} type="button">
                                <List className="h-4 w-4" />
                            </button>
                        </div>
                    </div>

                    {/* Advanced Filters Panel */}
                    <Collapsible onOpenChange={setFiltersOpen} open={filtersOpen}>
                        <CollapsibleContent>
                            <div className="rounded-lg border bg-card/50 p-4">
                                <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
                                    {/* UID Search */}
                                    <div className="space-y-2">
                                        <Label htmlFor="uid-search">Search by UID</Label>
                                        <Input id="uid-search" onChange={(e) => handleSearchInput(e.target.value, "uid")} placeholder="Enter exact UID..." value={uid} />
                                    </div>

                                    {/* Level Range */}
                                    <div className="space-y-2">
                                        <Label>Level Range</Label>
                                        <div className="flex items-center gap-2">
                                            <Input className="w-20" max={120} min={1} onChange={(e) => setLevelMin(e.target.value)} placeholder="Min" type="number" value={levelMin} />
                                            <span className="text-muted-foreground">-</span>
                                            <Input className="w-20" max={120} min={1} onChange={(e) => setLevelMax(e.target.value)} placeholder="Max" type="number" value={levelMax} />
                                            <Button onClick={handleLevelFilter} size="sm" variant="secondary">
                                                Apply
                                            </Button>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        </CollapsibleContent>
                    </Collapsible>

                    {/* Active Filters Pills */}
                    <AnimatePresence>
                        {hasActiveFilters && (
                            <motion.div animate={{ opacity: 1, y: 0 }} className="flex flex-wrap gap-2" exit={{ opacity: 0, y: -10 }} initial={{ opacity: 0, y: -10 }}>
                                {nickname && (
                                    <FilterPill
                                        label={`Name: ${nickname}`}
                                        onRemove={() => {
                                            setNickname("");
                                            void updateSearch({ nickname: undefined });
                                        }}
                                    />
                                )}
                                {uid && (
                                    <FilterPill
                                        label={`UID: ${uid}`}
                                        onRemove={() => {
                                            setUid("");
                                            void updateSearch({ uid: undefined });
                                        }}
                                    />
                                )}
                                {currentServer !== "all" && <FilterPill label={`Server: ${SERVERS.find((s) => s.value === currentServer)?.label || currentServer}`} onRemove={() => void updateSearch({ server: undefined })} />}
                                {currentGrade !== "all" && <FilterPill label={`Grade: ${currentGrade}`} onRemove={() => void updateSearch({ grade: undefined })} />}
                                {(levelMin || levelMax) && (
                                    <FilterPill
                                        label={`Level: ${levelMin || "1"}-${levelMax || "120"}`}
                                        onRemove={() => {
                                            setLevelMin("");
                                            setLevelMax("");
                                            void updateSearch({ levelMin: undefined, levelMax: undefined });
                                        }}
                                    />
                                )}
                            </motion.div>
                        )}
                    </AnimatePresence>
                </div>
            </InView>

            {/* Results Count */}
            <div className="flex items-center justify-between">
                <p className="text-muted-foreground text-sm">
                    {data.pagination.total > 0 ? (
                        <>
                            Showing {currentOffset + 1}-{Math.min(currentOffset + limit, data.pagination.total)} of <span className="font-medium text-foreground">{data.pagination.total.toLocaleString()}</span> players
                        </>
                    ) : (
                        "No players found"
                    )}
                </p>
            </div>

            {/* Results */}
            <div className={cn("transition-opacity duration-200", isLoading && "opacity-60")}>
                <AnimatePresence mode="wait">
                    {data.results.length > 0 ? (
                        <motion.div animate={{ opacity: 1, y: 0 }} exit={{ opacity: 0, y: -10 }} initial={{ opacity: 0, y: 10 }} key={`${viewMode}-${data.pagination.offset}-${data.pagination.total}`} transition={{ duration: 0.2, ease: "easeOut" }}>
                            {viewMode === "grid" ? <SearchResultsGrid results={data.results} /> : <SearchResultsList results={data.results} />}
                        </motion.div>
                    ) : (
                        <motion.div animate={{ opacity: 1 }} exit={{ opacity: 0 }} initial={{ opacity: 0 }} key="empty" transition={{ duration: 0.2 }}>
                            <EmptyState hasFilters={hasActiveFilters} onReset={handleResetFilters} />
                        </motion.div>
                    )}
                </AnimatePresence>
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
