import { useQuery } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import { ChevronRight } from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import { Pagination } from "#/components/operators/list/impl/components/Pagination";
import { useAuth } from "#/hooks/use-auth";
import { useDebounce } from "#/hooks/use-debounce";
import { leaderboardMoversQueryOptions, leaderboardQueryOptions, playerStandingQueryOptions } from "#/lib/api/user";
// import { leaderboardDistributionQueryOptions } from "#/lib/api/user"; // hidden: server share
import { formatNumber } from "#/lib/utils";
import { Route } from "#/routes/user.leaderboard";
import { Hero } from "./impl/components/Hero";
import { LeaderboardTable, LeaderboardTableSkeleton } from "./impl/components/LeaderboardTable";
import { MoversCard } from "./impl/components/MoversCard";
// import { ServerSplitCard } from "./impl/components/ServerSplitCard"; // hidden: server share
import { Toolbar } from "./impl/components/Toolbar";
import { YouCard } from "./impl/components/YouCard";
import { INTERVALS, type LeaderboardInterval, type LeaderboardScope, PAGE_SIZE, SERVERS, type ServerCode } from "./impl/constants";
import type { LeaderboardEntry } from "./impl/types";

export function Leaderboard() {
    const search = Route.useSearch();
    const navigate = useNavigate({ from: "/user/leaderboard" });
    const { user } = useAuth();

    const [inputValue, setInputValue] = useState(search.q);
    const debouncedQuery = useDebounce(inputValue.trim(), 300);
    const scope: LeaderboardScope = search.scope;
    const server: ServerCode | "All" = search.server;
    const interval: LeaderboardInterval = search.interval;
    const movementOnly = search.movement;
    const page = search.page;

    useEffect(() => {
        const next = inputValue.trim();
        if (next === search.q) return;
        navigate({ search: { ...search, q: next, page: 1 }, replace: true, resetScroll: false });
    }, [inputValue, navigate, search]);

    const apiServer = server === "All" ? undefined : server;
    const apiQuery = debouncedQuery || undefined;
    const offset = (page - 1) * PAGE_SIZE;

    const pageQuery = useQuery(
        leaderboardQueryOptions({
            server: apiServer,
            movement_interval: interval,
            movement_only: movementOnly,
            q: apiQuery,
            limit: PAGE_SIZE,
            offset,
        }),
    );
    const topQuery = useQuery(leaderboardQueryOptions({ server: apiServer, limit: 3, offset: 0 }));
    const moversQuery = useQuery(leaderboardMoversQueryOptions({ direction: "up", interval, limit: 3, server: apiServer }));
    const intervalMeta = INTERVALS.find((i) => i.value === interval) ?? INTERVALS[0];

    const standingQuery = useQuery({
        ...playerStandingQueryOptions({ uid: user?.uid ?? "", server: user?.server ?? "", interval }),
        enabled: Boolean(user?.uid && user?.server),
    });

    const updatedAt = pageQuery.data?.updated_at ?? null;
    const totalEntries = pageQuery.data?.total ?? 0;
    const totalPages = Math.max(1, Math.ceil(totalEntries / PAGE_SIZE));
    const isLoading = pageQuery.isLoading || pageQuery.isFetching;

    const visibleEntries = useMemo<LeaderboardEntry[]>(() => {
        const list = pageQuery.data?.entries ?? [];
        return list.map((entry) => ({
            ...entry,
            isSelf: Boolean(user && entry.uid === user.uid && entry.server === user.server),
        }));
    }, [pageQuery.data?.entries, user]);

    const topThree: LeaderboardEntry[] = topQuery.data?.entries ?? [];
    const referenceScore = topThree[0]?.total_score ?? null;
    const start = totalEntries === 0 ? 0 : offset + 1;
    const end = Math.min(offset + visibleEntries.length, totalEntries);

    const handlePageChange = (next: number) => {
        navigate({ search: { ...search, page: next }, replace: false, resetScroll: false });
        if (typeof window !== "undefined") {
            window.scrollTo({ top: 0, behavior: "smooth" });
        }
    };

    const handleScope = (next: LeaderboardScope) => {
        navigate({ search: { ...search, scope: next, page: 1 }, replace: true, resetScroll: false });
    };

    const handleServer = (next: ServerCode | "All") => {
        navigate({ search: { ...search, server: next, page: 1 }, replace: true, resetScroll: false });
    };

    const handleInterval = (next: LeaderboardInterval) => {
        navigate({ search: { ...search, interval: next, page: 1 }, replace: true, resetScroll: false });
    };

    const handleMovementOnly = (next: boolean) => {
        navigate({ search: { ...search, movement: next, page: 1 }, replace: true, resetScroll: false });
    };

    return (
        <div className="relative z-1 mx-auto w-[min(1280px,calc(100%-2rem))] pb-20">
            <div className="pt-7 pb-2.5">
                <nav aria-label="Breadcrumb" className="mb-2.5 flex items-center gap-1.5 font-medium font-sans text-[12px] text-muted-foreground leading-none">
                    <span>Doctors</span>
                    <ChevronRight className="h-2.5 w-2.5" aria-hidden />
                    <span className="text-foreground">Leaderboard</span>
                </nav>
            </div>

            <Hero rankedDoctors={totalEntries || null} topScore={referenceScore} updatedAt={updatedAt} isLoading={topQuery.isLoading} />

            <div className="mt-4 grid grid-cols-1 gap-5 lg:grid-cols-[minmax(0,1fr)_320px] lg:gap-6">
                <div className="flex min-w-0 flex-col gap-4">
                    <Toolbar scope={scope} onScope={handleScope} server={server} onServer={handleServer} interval={interval} onInterval={handleInterval} movementOnly={movementOnly} onMovementOnly={handleMovementOnly} query={inputValue} onQuery={setInputValue} />

                    <div className="overflow-hidden rounded-2xl border border-border bg-card shadow-[0_1px_2px_rgb(0_0_0/0.04)]">
                        {isLoading && visibleEntries.length === 0 ? <LeaderboardTableSkeleton /> : <LeaderboardTable entries={visibleEntries} referenceScore={referenceScore} isLoading={isLoading} intervalLabel={intervalMeta.since} />}
                        <div className="flex flex-wrap items-center justify-between gap-3 border-border border-t bg-[color-mix(in_srgb,var(--muted)_30%,transparent)] px-4 py-3.5">
                            <span className="font-mono text-muted-foreground text-xs tabular-nums leading-none">
                                Showing {start}-{end} of {formatNumber(totalEntries)} {totalEntries === 1 ? "Doctor" : "Doctors"}
                            </span>
                            <Pagination currentPage={page} totalPages={totalPages} onPageChange={handlePageChange} className="mt-0" />
                        </div>
                    </div>
                </div>

                <div className="flex flex-col gap-4 lg:gap-5">
                    {user ? <YouCard standing={standingQuery.data ?? null} rankedDoctors={totalEntries || null} /> : null}
                    <MoversCard movers={moversQuery.data ?? []} isLoading={moversQuery.isLoading} intervalLabel={intervalMeta.subtitle} />
                    {/* {distributionQuery.data ? <ServerSplitCard shares={distributionQuery.data} /> : null} */}
                </div>
            </div>
        </div>
    );
}

export { SERVERS };
