"use client";

import { ChevronLeft, ChevronRight, Grid, List, RotateCcw, Search, Users } from "lucide-react";
import { AnimatePresence, motion } from "motion/react";
import { useRouter } from "next/router";
import { useCallback, useEffect, useRef, useState } from "react";
import { InView } from "~/components/ui/motion-primitives/in-view";
import { Button } from "~/components/ui/shadcn/button";
import { Input } from "~/components/ui/shadcn/input";
import { clearSearchAbortController, fetchSearchResultsCached, getSearchAbortController } from "~/lib/search-utils";
import { cn } from "~/lib/utils";
import type { SearchQuery, SearchResponse } from "~/types/api";
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
    const searchInputRef = useRef<HTMLInputElement>(null);

    const [query, setQuery] = useState((router.query.q as string) || "");
    const [currentOffset, setCurrentOffset] = useState(Number(router.query.offset) || 0);

    const limit = data.pagination.limit;
    const totalPages = Math.ceil(data.pagination.total / limit);
    const currentPage = Math.floor(currentOffset / limit) + 1;

    const hasActiveFilters = !!query;

    const updateSearch = useCallback(async (newQ?: string, newOffset?: number) => {
        setIsLoading(true);

        const searchQ = newQ !== undefined ? newQ : query;
        const searchOffset = newOffset !== undefined ? newOffset : 0;

        try {
            const controller = getSearchAbortController();

            const searchQuery: SearchQuery = {
                q: searchQ || undefined,
                limit: 24,
                offset: searchOffset,
            };

            const result = await fetchSearchResultsCached(searchQuery, { signal: controller.signal });
            setData(result);
            clearSearchAbortController();

            // Update URL
            const newSearchParams = new URLSearchParams();
            if (searchQ) newSearchParams.set("q", searchQ);
            if (searchOffset > 0) newSearchParams.set("offset", String(searchOffset));

            const newURL = `${window.location.pathname}${newSearchParams.toString() ? `?${newSearchParams.toString()}` : ""}`;
            window.history.replaceState({ ...window.history.state, as: newURL, url: newURL }, "", newURL);
        } catch (error) {
            if (error instanceof Error && error.name !== "AbortError") {
                console.error("Search failed:", error);
            }
        } finally {
            setIsLoading(false);
        }
    }, [query]);

    const searchTimeoutRef = useRef<NodeJS.Timeout | null>(null);

    const handleSearchInput = useCallback(
        (value: string) => {
            setQuery(value);
            setCurrentOffset(0);

            if (searchTimeoutRef.current) {
                clearTimeout(searchTimeoutRef.current);
            }
            searchTimeoutRef.current = setTimeout(() => {
                void updateSearch(value, 0);
            }, 300);
        },
        [updateSearch],
    );

    const handleResetFilters = () => {
        setQuery("");
        setCurrentOffset(0);
        void updateSearch("", 0);
    };

    const handlePageChange = (newOffset: number) => {
        setCurrentOffset(newOffset);
        void updateSearch(undefined, newOffset);
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
                            <p className="text-muted-foreground">Find Arknights players by name or UID</p>
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
                <div className="space-y-3">
                    <div className="relative">
                        <Search className="absolute top-1/2 left-3 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
                        <Input className="h-12 pr-20 pl-10 text-base" onChange={(e) => handleSearchInput(e.target.value)} placeholder="Search by player name or UID..." ref={searchInputRef} value={query} />
                        <kbd className="pointer-events-none absolute top-1/2 right-3 hidden -translate-y-1/2 select-none items-center gap-1 rounded border bg-muted px-1.5 py-0.5 font-medium font-mono text-muted-foreground text-xs sm:flex">
                            <span className="text-xs">⌘</span>K
                        </kbd>
                    </div>

                    {/* Active Filter + View Toggle Row */}
                    <div className="flex flex-wrap items-center gap-2">
                        <AnimatePresence>
                            {hasActiveFilters && (
                                <motion.div animate={{ opacity: 1, scale: 1 }} className="flex items-center gap-2" exit={{ opacity: 0, scale: 0.9 }} initial={{ opacity: 0, scale: 0.9 }}>
                                    <FilterPill
                                        label={`Search: ${query}`}
                                        onRemove={handleResetFilters}
                                    />
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
