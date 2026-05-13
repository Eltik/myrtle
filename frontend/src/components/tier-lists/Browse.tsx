import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import { useCallback, useEffect, useMemo, useState } from "react";
import { Skeleton } from "#/components/ui/skeleton";
import { useAuth } from "#/hooks/use-auth";
import { useDebounce } from "#/hooks/use-debounce";
import { browseTierListsQueryOptions, type ITierListBrowseItem, recordTierListViewFn } from "#/lib/api/tier-lists";
import { Route } from "#/routes/tier-lists";
import BrowseCard from "./BrowseCard";
import { FilterToolbar, type FlairOption, type TierListSort, type TierListType } from "./FilterToolbar";
import { Hero } from "./Hero";
import { OfficialRail } from "./OfficialRail";
import { TrendingStrip } from "./TrendingStrip";

const PAGE_INCREMENT = 24;
const INITIAL_PAGE_SIZE = 24;
const TRENDING_LIMIT = 3;
const OFFICIAL_RAIL_LIMIT = 8;

function sortLists(lists: ITierListBrowseItem[], sort: TierListSort): ITierListBrowseItem[] {
    const arr = [...lists];
    switch (sort) {
        case "trending":
            return arr.sort((a, b) => {
                if (a.isTrending !== b.isTrending) return a.isTrending ? -1 : 1;
                if (a.trendingScore !== b.trendingScore) return b.trendingScore - a.trendingScore;
                return b.views24h - a.views24h;
            });
        case "recent":
            return arr.sort((a, b) => b.updatedAtMs - a.updatedAtMs);
        case "newest":
            return arr.sort((a, b) => b.createdAtMs - a.createdAtMs);
        case "views":
            return arr.sort((a, b) => b.views - a.views);
        case "favorites":
            return arr.sort((a, b) => b.favorites - a.favorites);
        case "shares":
            return arr.sort((a, b) => b.shares - a.shares);
        default:
            return arr;
    }
}

function matchesQuery(list: ITierListBrowseItem, q: string): boolean {
    if (!q) return true;
    const needle = q.toLowerCase();
    if (list.title.toLowerCase().includes(needle)) return true;
    if (list.description.toLowerCase().includes(needle)) return true;
    if (list.author.name.toLowerCase().includes(needle)) return true;
    if (list.flairLabel?.toLowerCase().includes(needle)) return true;
    return false;
}

export function Browse() {
    const queryClient = useQueryClient();
    const navigate = useNavigate({ from: "/tier-lists" });
    const search = Route.useSearch();
    const { user } = useAuth();

    const { data, isLoading, isError, refetch, isFetching } = useQuery(browseTierListsQueryOptions());
    const allLists = useMemo<ITierListBrowseItem[]>(() => data ?? [], [data]);

    const [inputQuery, setInputQuery] = useState(search.q);
    const debouncedQuery = useDebounce(inputQuery.trim(), 200);
    const [visibleCount, setVisibleCount] = useState(INITIAL_PAGE_SIZE);

    useEffect(() => {
        if (debouncedQuery === search.q) return;
        navigate({ search: { ...search, q: debouncedQuery }, replace: true, resetScroll: false });
    }, [debouncedQuery, navigate, search]);

    useEffect(() => {
        setVisibleCount(INITIAL_PAGE_SIZE);
    }, []);

    const recordView = useMutation({
        mutationFn: (slug: string) => recordTierListViewFn({ data: slug }),
        onSuccess: () => queryClient.invalidateQueries({ queryKey: ["tier-lists", "browse"] }),
    });

    const handleOpen = useCallback(
        (slug: string) => {
            recordView.mutate(slug);
        },
        [recordView],
    );

    const flairOptions = useMemo<FlairOption[]>(() => {
        const seen = new Map<string, FlairOption>();
        for (const list of allLists) {
            if (list.flairCode && !seen.has(list.flairCode)) {
                seen.set(list.flairCode, { code: list.flairCode, label: list.flairLabel ?? list.flairCode, color: list.flairColor });
            }
        }
        return Array.from(seen.values()).sort((a, b) => a.label.localeCompare(b.label));
    }, [allLists]);

    const officialFeatured = useMemo(
        () =>
            sortLists(
                allLists.filter((l) => l.listType === "official"),
                "trending",
            ).slice(0, OFFICIAL_RAIL_LIMIT),
        [allLists],
    );

    const trendingExplicit = useMemo(
        () =>
            sortLists(
                allLists.filter((l) => l.isTrending),
                "trending",
            ).slice(0, TRENDING_LIMIT),
        [allLists],
    );
    const trendingFallback = useMemo(() => sortLists(allLists, "trending").slice(0, TRENDING_LIMIT), [allLists]);
    const trendingTop = trendingExplicit.length > 0 ? trendingExplicit : trendingFallback;
    const isTrendingMode = trendingExplicit.length > 0;

    const filtered = useMemo(() => {
        return allLists.filter((list) => {
            if (search.type !== "all" && list.listType !== search.type) return false;
            if (search.flair.length > 0 && (!list.flairCode || !search.flair.includes(list.flairCode))) return false;
            if (!matchesQuery(list, debouncedQuery)) return false;
            return true;
        });
    }, [allLists, search.type, search.flair, debouncedQuery]);

    const sorted = useMemo(() => sortLists(filtered, search.sort), [filtered, search.sort]);
    const visible = useMemo(() => sorted.slice(0, visibleCount), [sorted, visibleCount]);
    const hasMore = sorted.length > visibleCount;

    const setType = (next: TierListType) => navigate({ search: { ...search, type: next }, replace: true, resetScroll: false });
    const setSort = (next: TierListSort) => navigate({ search: { ...search, sort: next }, replace: true, resetScroll: false });

    const toggleFlair = (code: string) => {
        const next = search.flair.includes(code) ? search.flair.filter((c) => c !== code) : [...search.flair, code];
        navigate({ search: { ...search, flair: next }, replace: true, resetScroll: false });
    };

    const clearFlairs = () => navigate({ search: { ...search, flair: [] }, replace: true, resetScroll: false });

    const clearAllFilters = () => {
        setInputQuery("");
        navigate({ search: { type: "all", sort: "trending", q: "", flair: [] }, replace: true, resetScroll: false });
    };

    const scrollToGrid = () => {
        if (typeof window === "undefined") return;
        const el = document.getElementById("tier-lists-grid");
        if (!el) return;
        const offset = el.getBoundingClientRect().top + window.scrollY - 100;
        window.scrollTo({ top: offset, behavior: "smooth" });
    };

    return (
        <main className="min-h-dvh pb-24">
            <Hero total={allLists.length} canCreate={Boolean(user)} />

            {!isLoading && !isError && (
                <>
                    <OfficialRail
                        lists={officialFeatured}
                        onOpen={handleOpen}
                        onViewAll={() => {
                            setType("official");
                            scrollToGrid();
                        }}
                    />
                    <TrendingStrip lists={trendingTop} onOpen={handleOpen} isTrendingMode={isTrendingMode} />
                </>
            )}

            <div id="tier-lists-grid" className="mx-auto mt-8 w-[min(1080px,calc(100%-2rem))] scroll-mt-32">
                <FilterToolbar
                    type={search.type}
                    sort={search.sort}
                    query={inputQuery}
                    selectedFlairs={search.flair}
                    flairOptions={flairOptions}
                    resultCount={sorted.length}
                    totalCount={allLists.length}
                    onTypeChange={setType}
                    onSortChange={(next) => {
                        setSort(next);
                        scrollToGrid();
                    }}
                    onQueryChange={setInputQuery}
                    onFlairToggle={toggleFlair}
                    onClearFlairs={clearFlairs}
                />

                <div className="mt-5">
                    {isLoading ? (
                        <div className="grid grid-cols-1 gap-3 md:grid-cols-2 lg:grid-cols-3">
                            {["s1", "s2", "s3", "s4", "s5", "s6", "s7", "s8", "s9"].map((k) => (
                                <Skeleton key={k} className="h-60 rounded-lg" />
                            ))}
                        </div>
                    ) : isError ? (
                        <div className="rounded-lg border border-dashed border-border bg-muted/20 px-5 py-10 text-center">
                            <p className="m-0 font-sans text-sm text-muted-foreground">Failed to load tier lists.</p>
                            <button
                                type="button"
                                onClick={() => refetch()}
                                disabled={isFetching}
                                className="mt-3 inline-flex h-8 cursor-pointer items-center gap-1.5 rounded-lg border border-border bg-popover px-3 font-sans text-xs font-medium leading-none text-foreground transition-colors hover:bg-accent disabled:opacity-60"
                            >
                                {isFetching ? "Retrying…" : "Retry"}
                            </button>
                        </div>
                    ) : sorted.length === 0 ? (
                        <div className="rounded-lg border border-dashed border-border bg-muted/20 px-5 py-12 text-center">
                            <p className="m-0 font-sans text-sm font-medium text-foreground">No lists match these filters.</p>
                            <p className="mt-1 font-sans text-[12.5px] text-muted-foreground">Try clearing flairs or switching the type filter.</p>
                            <button type="button" onClick={clearAllFilters} className="mt-4 inline-flex h-8 cursor-pointer items-center gap-1.5 rounded-lg border border-border bg-popover px-3 font-sans text-xs font-medium leading-none text-foreground transition-colors hover:bg-accent">
                                Clear filters
                            </button>
                        </div>
                    ) : (
                        <>
                            <div className="grid grid-cols-1 gap-3 md:grid-cols-2 lg:grid-cols-3">
                                {visible.map((tl) => (
                                    <BrowseCard key={tl.id} tl={tl} onOpen={handleOpen} />
                                ))}
                            </div>

                            {hasMore && (
                                <div className="mt-6 flex justify-center">
                                    <button
                                        type="button"
                                        onClick={() => setVisibleCount((n) => n + PAGE_INCREMENT)}
                                        className="inline-flex cursor-pointer items-center gap-2 rounded-lg border border-dashed border-border bg-transparent px-4 py-2.5 font-sans text-[12.5px] font-medium leading-none text-muted-foreground transition-colors hover:border-primary hover:bg-[color-mix(in_srgb,var(--primary)_5%,transparent)] hover:text-foreground"
                                    >
                                        <span className="h-1.5 w-1.5 rounded-full bg-primary shadow-[0_0_8px_color-mix(in_srgb,var(--primary)_50%,transparent)]" aria-hidden="true" />
                                        <span>Load {Math.min(PAGE_INCREMENT, sorted.length - visibleCount)} more</span>
                                        <span className="font-mono text-[10.5px] text-muted-foreground/70 tabular-nums">
                                            {visibleCount} / {sorted.length}
                                        </span>
                                    </button>
                                </div>
                            )}
                        </>
                    )}
                </div>
            </div>
        </main>
    );
}
