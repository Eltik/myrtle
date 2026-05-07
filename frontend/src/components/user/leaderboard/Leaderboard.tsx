import { useQuery } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import { ChevronRight } from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import { Pagination } from "#/components/operators/list/impl/components/Pagination";
import { useAuth } from "#/hooks/use-auth";
import { leaderboardDistributionQueryOptions, leaderboardMoversQueryOptions, leaderboardQueryOptions, playerStandingQueryOptions } from "#/lib/api/user";
import { formatNumber } from "#/lib/utils";
import { Route } from "#/routes/user.leaderboard";
import { useDebounce } from "../search/impl/useDebounce";
import { Hero } from "./impl/components/Hero";
import { LeaderboardTable, LeaderboardTableSkeleton } from "./impl/components/LeaderboardTable";
import { Medalists } from "./impl/components/Medalists";
import { MoversCard } from "./impl/components/MoversCard";
import { ServerSplitCard } from "./impl/components/ServerSplitCard";
import { Toolbar } from "./impl/components/Toolbar";
import { YouCard } from "./impl/components/YouCard";
import { type LeaderboardScope, PAGE_SIZE, SERVERS, type ServerCode } from "./impl/constants";
import type { LeaderboardEntry } from "./impl/types";

export function Leaderboard() {
    const search = Route.useSearch();
    const navigate = useNavigate({ from: "/user/leaderboard" });
    const { user } = useAuth();

    const [inputValue, setInputValue] = useState(search.q);
    const debouncedQuery = useDebounce(inputValue.trim(), 300).toLowerCase();
    const scope: LeaderboardScope = search.scope;
    const server: ServerCode | "All" = search.server;
    const page = search.page;

    useEffect(() => {
        const next = inputValue.trim();
        if (next === search.q) return;
        navigate({ search: { ...search, q: next, page: 1 }, replace: true });
    }, [inputValue, navigate, search]);

    const apiServer = server === "All" ? undefined : server;
    const offset = (page - 1) * PAGE_SIZE;

    const pageQuery = useQuery(leaderboardQueryOptions({ server: apiServer, limit: PAGE_SIZE, offset }));
    const topQuery = useQuery(leaderboardQueryOptions({ server: apiServer, limit: 3, offset: 0 }));
    const moversQuery = useQuery(leaderboardMoversQueryOptions({ direction: "up", interval: "7 days", limit: 3, server: apiServer }));
    const distributionQuery = useQuery(leaderboardDistributionQueryOptions({ top: 250 }));

    const standingQuery = useQuery({
        ...playerStandingQueryOptions({ uid: user?.uid ?? "", server: user?.server ?? "" }),
        enabled: Boolean(user?.uid && user?.server),
    });

    const isLoading = pageQuery.isLoading || pageQuery.isFetching;
    const totalEntries = pageQuery.data?.total ?? 0;
    const totalPages = Math.max(1, Math.ceil(totalEntries / PAGE_SIZE));
    const updatedAt = pageQuery.data?.updated_at ?? null;

    const visibleEntries = useMemo<LeaderboardEntry[]>(() => {
        const list = pageQuery.data?.entries ?? [];
        const matchSelf = (entry: LeaderboardEntry) => Boolean(user && entry.uid === user.uid && entry.server === user.server);
        const flagged = list.map((entry) => ({ ...entry, isSelf: matchSelf(entry) }));
        if (!debouncedQuery) return flagged;
        return flagged.filter((entry) => {
            const nick = (entry.nickname ?? "").toLowerCase();
            return nick.includes(debouncedQuery) || entry.uid.includes(debouncedQuery);
        });
    }, [pageQuery.data?.entries, debouncedQuery, user]);

    const topThree: LeaderboardEntry[] = topQuery.data?.entries ?? [];
    const referenceScore = topThree[0]?.total_score ?? null;
    const start = totalEntries === 0 ? 0 : offset + 1;
    const end = Math.min(offset + (pageQuery.data?.entries.length ?? 0), totalEntries);

    const handlePageChange = (next: number) => {
        navigate({ search: { ...search, page: next }, replace: false });
        if (typeof window !== "undefined") {
            window.scrollTo({ top: 0, behavior: "smooth" });
        }
    };

    const handleScope = (next: LeaderboardScope) => {
        navigate({ search: { ...search, scope: next, page: 1 }, replace: true });
    };

    const handleServer = (next: ServerCode | "All") => {
        navigate({ search: { ...search, server: next, page: 1 }, replace: true });
    };

    return (
        <div className="relative z-1 mx-auto w-[min(1280px,calc(100%-2rem))] pb-20">
            <div className="pt-7 pb-2.5">
                <nav aria-label="Breadcrumb" className="mb-2.5 flex items-center gap-1.5 font-sans text-[12px] font-medium leading-none text-muted-foreground">
                    <span>Doctors</span>
                    <ChevronRight className="h-2.5 w-2.5" aria-hidden />
                    <span className="text-foreground">Leaderboard</span>
                </nav>
            </div>

            <Hero rankedDoctors={totalEntries || null} topScore={referenceScore} updatedAt={updatedAt} isLoading={topQuery.isLoading} />

            <div className="mt-4 grid grid-cols-1 gap-5 lg:grid-cols-[minmax(0,1fr)_320px] lg:gap-6">
                <div className="flex min-w-0 flex-col gap-4">
                    <Medalists entries={topThree} isLoading={topQuery.isLoading} />

                    <Toolbar scope={scope} onScope={handleScope} server={server} onServer={handleServer} query={inputValue} onQuery={setInputValue} />

                    <div className="overflow-hidden rounded-2xl border border-border bg-card shadow-[0_1px_2px_rgb(0_0_0/0.04)]">
                        {isLoading && visibleEntries.length === 0 ? <LeaderboardTableSkeleton /> : <LeaderboardTable entries={visibleEntries} referenceScore={referenceScore} isLoading={isLoading} />}
                        <div className="flex flex-wrap items-center justify-between gap-3 border-t border-border bg-[color-mix(in_srgb,var(--muted)_30%,transparent)] px-4 py-3.5">
                            <span className="font-mono text-xs leading-none text-muted-foreground tabular-nums">
                                Showing {start}–{end} of {formatNumber(totalEntries)} {totalEntries === 1 ? "Doctor" : "Doctors"}
                            </span>
                            <Pagination currentPage={page} totalPages={totalPages} onPageChange={handlePageChange} className="mt-0" />
                        </div>
                    </div>
                </div>

                <div className="flex flex-col gap-4 lg:gap-5">
                    {user ? <YouCard standing={standingQuery.data ?? null} rankedDoctors={totalEntries || null} /> : null}
                    <MoversCard movers={moversQuery.data ?? []} isLoading={moversQuery.isLoading} />
                    {distributionQuery.data ? <ServerSplitCard shares={distributionQuery.data} /> : null}
                </div>
            </div>
        </div>
    );
}

export { SERVERS };
