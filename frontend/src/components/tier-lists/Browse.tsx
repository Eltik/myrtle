import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import { useCallback, useEffect, useMemo, useState } from "react";
import { Skeleton } from "#/components/ui/skeleton";
import { useAuth } from "#/hooks/use-auth";
import { useDebounce } from "#/hooks/use-debounce";
import { browseTierListsQueryOptions, favoritedTierListsQueryOptions, type ITierListBrowseItem, recordTierListViewFn, tierListFlairsQueryOptions } from "#/lib/api/tier-lists";
import { Route } from "#/routes/tier-lists";
import BrowseCard from "./BrowseCard";
import { FilterToolbar, type IFlairOption, type TierListSort, type TierListType } from "./FilterToolbar";
import { Hero } from "./Hero";
import { OfficialRail } from "./OfficialRail";
import { matchesBrowseQuery, sortBrowseItems } from "./shared";

const PAGE_INCREMENT = 24;
const INITIAL_PAGE_SIZE = 24;
const OFFICIAL_RAIL_LIMIT = 8;

export function Browse() {
    const queryClient = useQueryClient();
    const navigate = useNavigate({ from: "/tier-lists" });
    const search = Route.useSearch();
    const { user } = useAuth();
    const authed = Boolean(user);

    useEffect(() => {
        if (!authed && search.type === "favorites") {
            navigate({ search: { ...search, type: "all" }, replace: true, resetScroll: false });
        }
    }, [authed, search, navigate]);

    const browseQuery = useQuery(browseTierListsQueryOptions());
    const favoritesQuery = useQuery(favoritedTierListsQueryOptions(authed));
    const viewingFavorites = authed && search.type === "favorites";

    const { data, isLoading, isError, refetch, isFetching } = viewingFavorites ? favoritesQuery : browseQuery;
    const { data: flairCatalog } = useQuery(tierListFlairsQueryOptions());
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

    const flairOptions = useMemo<IFlairOption[]>(() => {
        const counts = new Map<string, number>();
        for (const list of allLists) {
            if (list.flairCode) counts.set(list.flairCode, (counts.get(list.flairCode) ?? 0) + 1);
        }

        if (flairCatalog && flairCatalog.length > 0) {
            return flairCatalog
                .filter((f) => f.isActive)
                .map<IFlairOption>((f) => ({
                    code: f.code,
                    label: f.label,
                    color: f.color,
                    count: counts.get(f.code) ?? 0,
                    displayOrder: f.displayOrder,
                }))
                .sort((a, b) => (a.displayOrder ?? 0) - (b.displayOrder ?? 0) || a.label.localeCompare(b.label));
        }

        const seen = new Map<string, IFlairOption>();
        for (const list of allLists) {
            if (list.flairCode && !seen.has(list.flairCode)) {
                seen.set(list.flairCode, { code: list.flairCode, label: list.flairLabel ?? list.flairCode, color: list.flairColor, count: counts.get(list.flairCode) ?? 0 });
            }
        }
        return Array.from(seen.values()).sort((a, b) => a.label.localeCompare(b.label));
    }, [allLists, flairCatalog]);

    const officialFeatured = useMemo(
        () =>
            sortBrowseItems(
                allLists.filter((l) => l.listType === "official"),
                "trending",
            ).slice(0, OFFICIAL_RAIL_LIMIT),
        [allLists],
    );

    const filtered = useMemo(() => {
        return allLists.filter((list) => {
            if (!viewingFavorites && search.type !== "all" && list.listType !== search.type) return false;
            if (search.flair.length > 0 && (!list.flairCode || !search.flair.includes(list.flairCode))) return false;
            if (!matchesBrowseQuery(list, debouncedQuery)) return false;
            return true;
        });
    }, [allLists, search.type, search.flair, debouncedQuery, viewingFavorites]);

    const sorted = useMemo(() => sortBrowseItems(filtered, search.sort), [filtered, search.sort]);
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
        navigate({ search: { type: "all", sort: "recent", q: "", flair: [] }, replace: true, resetScroll: false });
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

            {!isLoading && !isError && !viewingFavorites && (
                <OfficialRail
                    lists={officialFeatured}
                    onOpen={handleOpen}
                    onViewAll={() => {
                        setType("official");
                        scrollToGrid();
                    }}
                />
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
                    showFavoritesTab={authed}
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
                        <div className="rounded-lg border border-border border-dashed bg-muted/20 px-5 py-10 text-center">
                            <p className="m-0 font-sans text-muted-foreground text-sm">Failed to load tier lists.</p>
                            <button
                                type="button"
                                onClick={() => refetch()}
                                disabled={isFetching}
                                className="mt-3 inline-flex h-8 cursor-pointer items-center gap-1.5 rounded-lg border border-border bg-popover px-3 font-medium font-sans text-foreground text-xs leading-none transition-colors hover:bg-accent disabled:opacity-60"
                            >
                                {isFetching ? "Retrying…" : "Retry"}
                            </button>
                        </div>
                    ) : sorted.length === 0 ? (
                        viewingFavorites && allLists.length === 0 ? (
                            <div className="rounded-lg border border-border border-dashed bg-muted/20 px-5 py-12 text-center">
                                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" className="mx-auto h-7 w-7 text-muted-foreground/70" aria-hidden="true">
                                    <path d="M20.84 4.61a5.5 5.5 0 0 0-7.78 0L12 5.67l-1.06-1.06a5.5 5.5 0 0 0-7.78 7.78l1.06 1.06L12 21.23l7.78-7.78 1.06-1.06a5.5 5.5 0 0 0 0-7.78z" />
                                </svg>
                                <p className="mt-3 mb-0 font-medium font-sans text-foreground text-sm">No favorites yet</p>
                                <p className="mt-1 font-sans text-[12.5px] text-muted-foreground">Tap the heart on any tier list to save it here for quick access.</p>
                                <button type="button" onClick={() => setType("all")} className="mt-4 inline-flex h-8 cursor-pointer items-center gap-1.5 rounded-lg border border-border bg-popover px-3 font-medium font-sans text-foreground text-xs leading-none transition-colors hover:bg-accent">
                                    Browse tier lists
                                </button>
                            </div>
                        ) : (
                            <div className="rounded-lg border border-border border-dashed bg-muted/20 px-5 py-12 text-center">
                                <p className="m-0 font-medium font-sans text-foreground text-sm">No lists match these filters.</p>
                                <p className="mt-1 font-sans text-[12.5px] text-muted-foreground">Try clearing flairs or switching the type filter.</p>
                                <button type="button" onClick={clearAllFilters} className="mt-4 inline-flex h-8 cursor-pointer items-center gap-1.5 rounded-lg border border-border bg-popover px-3 font-medium font-sans text-foreground text-xs leading-none transition-colors hover:bg-accent">
                                    Clear filters
                                </button>
                            </div>
                        )
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
                                        className="inline-flex cursor-pointer items-center gap-2 rounded-lg border border-border border-dashed bg-transparent px-4 py-2.5 font-medium font-sans text-[12.5px] text-muted-foreground leading-none transition-colors hover:border-primary hover:bg-[color-mix(in_srgb,var(--primary)_5%,transparent)] hover:text-foreground"
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
