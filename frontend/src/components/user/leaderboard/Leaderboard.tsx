import { useNavigate } from "@tanstack/react-router";
import { ChevronRight } from "lucide-react";
import { useEffect, useState } from "react";
import { Pagination } from "#/components/operators/list/impl/components/Pagination";
import { useAuth } from "#/hooks/use-auth";
import { useDebounce } from "#/hooks/use-debounce";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { Route } from "#/routes/user.leaderboard";
import { Hero } from "./impl/components/Hero";
import { MostHeldCard } from "./impl/components/inventory/MostHeldCard";
import { StandingCard } from "./impl/components/inventory/StandingCard";
import { LeaderboardTable, LeaderboardTableSkeleton } from "./impl/components/LeaderboardTable";
import { MoversCard } from "./impl/components/MoversCard";
// import { ServerSplitCard } from "./impl/components/ServerSplitCard"; // hidden: server share
import { Toolbar } from "./impl/components/Toolbar";
import { YouCard } from "./impl/components/YouCard";
import { INTERVALS, PAGE_SIZE, type Ranking, SERVERS } from "./impl/constants";
import { useLeaderboardData } from "./impl/useLeaderboardData";
import type { messages } from "./Leaderboard.messages";

type Search = ReturnType<typeof Route.useSearch>;

export function Leaderboard() {
    const t: TypedT<typeof messages> = useT("user");
    const search = Route.useSearch();
    const navigate = useNavigate({ from: "/user/leaderboard" });
    const { user } = useAuth();

    const [inputValue, setInputValue] = useState(search.q);
    const debouncedQuery = useDebounce(inputValue.trim(), 300);
    const { scope, server, interval, sort, movement: movementOnly, page } = search;
    // One page, one table: `item` in the URL switches what the table is
    // ranked by, nothing else about the page changes shape.
    const ranking: Ranking = search.item ? { kind: "item", item: search.item } : { kind: "score", sort };
    const byItem = ranking.kind === "item";
    const offset = (page - 1) * PAGE_SIZE;
    const intervalMeta = INTERVALS.find((i) => i.value === interval) ?? INTERVALS[0];

    /** Every filter change resets to page 1 and replaces history; paging itself pushes. */
    const setSearch = (patch: Partial<Search>, { push = false } = {}) => {
        navigate({ search: { ...search, page: 1, ...patch }, replace: !push, resetScroll: false });
    };

    useEffect(() => {
        const next = inputValue.trim();
        if (next === search.q) return;
        navigate({ search: { ...search, q: next, page: 1 }, replace: true, resetScroll: false });
    }, [inputValue, navigate, search]);

    const data = useLeaderboardData({
        ranking,
        sort,
        server: server === "All" ? undefined : server,
        q: debouncedQuery || undefined,
        interval,
        movementOnly,
        offset,
        me: user ? { uid: user.uid, server: user.server } : null,
    });

    const totalPages = Math.max(1, Math.ceil(data.total / PAGE_SIZE));
    const start = data.total === 0 ? 0 : offset + 1;
    const end = Math.min(offset + data.rows.length, data.total);

    const handlePageChange = (next: number) => {
        setSearch({ page: next }, { push: true });
        if (typeof window !== "undefined") {
            window.scrollTo({ top: 0, behavior: "smooth" });
        }
    };
    const handleRanking = (next: Ranking) => {
        setSearch(next.kind === "item" ? { item: next.item } : { item: "", sort: next.sort });
    };

    const youCard = user ? (
        byItem && data.currentItem ? (
            <StandingCard item={data.currentItem} me={{ uid: user.uid, server: user.server, nickname: user.nickname, avatar_id: user.avatar_id }} standing={data.itemStanding} isLoading={data.itemStandingLoading} />
        ) : (
            <YouCard standing={data.scoreStanding} rankedDoctors={data.population} />
        )
    ) : null;

    return (
        <div className="page-shell [--page-max:1280px]">
            <div className="pb-2.5">
                <nav aria-label={t("leaderboard.breadcrumb.label")} className="mb-2.5 flex items-center gap-1.5 font-medium font-sans text-[12px] text-muted-foreground leading-none">
                    <span>{t("leaderboard.breadcrumb.doctors")}</span>
                    <ChevronRight className="h-2.5 w-2.5" aria-hidden />
                    <span className="text-foreground">{t("leaderboard.breadcrumb.current")}</span>
                </nav>
            </div>

            <Hero item={data.currentItem} rankedDoctors={data.population} topValue={data.topValue} updatedAt={data.updatedAt} isLoading={data.heroLoading} />

            {/*
              One column below lg, two above. The "You" card is placed first in
              source so on a phone your own standing sits between the toolbar
              and the table, where it is read before scrolling; on desktop the
              explicit grid placement puts it at the top of the sidebar.
            */}
            <div className="mt-4 grid grid-cols-1 gap-4 lg:grid-cols-[minmax(0,1fr)_320px] lg:grid-rows-[auto_1fr] lg:gap-x-6 lg:gap-y-5">
                <div className="flex min-w-0 flex-col gap-4 lg:col-start-1 lg:row-span-2 lg:row-start-1">
                    <Toolbar
                        ranking={ranking}
                        onRanking={handleRanking}
                        catalog={data.catalog}
                        materials={data.materials}
                        scope={scope}
                        onScope={(next) => setSearch({ scope: next })}
                        server={server}
                        onServer={(next) => setSearch({ server: next })}
                        interval={interval}
                        onInterval={(next) => setSearch({ interval: next })}
                        movementOnly={movementOnly}
                        onMovementOnly={(next) => setSearch({ movement: next })}
                        query={inputValue}
                        onQuery={setInputValue}
                    />

                    {youCard ? <div className="lg:hidden">{youCard}</div> : null}

                    <div className="overflow-hidden rounded-2xl border border-border bg-card shadow-[0_1px_2px_rgb(0_0_0/0.04)]">
                        {data.isLoading && data.rows.length === 0 ? (
                            <LeaderboardTableSkeleton />
                        ) : (
                            <LeaderboardTable rows={data.rows} ranking={ranking} onRanking={handleRanking} catalog={data.catalog} materials={data.materials} topValue={byItem ? data.topValue : null} isLoading={data.isLoading} intervalKey={intervalMeta.sinceKey} />
                        )}
                        <div className="flex flex-wrap items-center justify-between gap-3 border-border border-t bg-[color-mix(in_srgb,var(--muted)_30%,transparent)] px-3 py-3 sm:px-4 sm:py-3.5">
                            <span className="font-mono text-muted-foreground text-xs tabular-nums leading-none">{t(byItem ? "leaderboard.showingHolders" : "leaderboard.showing", { start, end, count: data.total })}</span>
                            <Pagination currentPage={page} totalPages={totalPages} onPageChange={handlePageChange} className="mt-0 w-auto" />
                        </div>
                    </div>
                </div>

                {youCard ? <div className="hidden lg:col-start-2 lg:row-start-1 lg:block">{youCard}</div> : null}
                <div className={youCard ? "flex flex-col gap-4 lg:col-start-2 lg:row-start-2 lg:gap-5" : "flex flex-col gap-4 lg:col-start-2 lg:row-span-2 lg:row-start-1 lg:gap-5"}>
                    {byItem ? <MostHeldCard catalog={data.catalog} current={ranking.item} onItem={(item) => handleRanking({ kind: "item", item })} isLoading={data.catalogLoading} /> : <MoversCard movers={data.movers} isLoading={data.moversLoading} intervalKey={intervalMeta.subtitleKey} />}
                    {/* {distributionQuery.data ? <ServerSplitCard shares={distributionQuery.data} /> : null} */}
                </div>
            </div>
        </div>
    );
}

export { SERVERS };
