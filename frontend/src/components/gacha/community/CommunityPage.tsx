import { useQuery } from "@tanstack/react-query";
import { useMemo } from "react";
import { LoadFailed } from "#/components/ui/load-failed";
import { useAuth } from "#/hooks/use-auth";
import { bannersQueryOptions, gachaEnhancedStatsQueryOptions, type IBanner, type IBannerPullStat, myGachaStatsQueryOptions, PULLS_GAMEDATA_SERVER, perBannerStatsQueryOptions } from "#/lib/api/gacha";
import { operatorsIndexQueryOptions } from "#/lib/api/operators";
import { useGamedataServer, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { IOperatorIndexEntry } from "#/types/operators";
import type { messages } from "./CommunityPage.messages";
import { BannerRunsPanel } from "./impl/BannerRunsPanel";
import styles from "./impl/CommunityPage.module.css";
import { KpiStrip } from "./impl/KpiStrip";
import { Leaderboard } from "./impl/Leaderboard";
import { PageHeader } from "./impl/PageHeader";
import { type IPersonalRarityRates, RarityPanel } from "./impl/RarityPanel";
import { TimingPanel } from "./impl/TimingPanel";

export function CommunityPage() {
    const t: TypedT<typeof messages> = useT("gacha");
    const enhanced = useQuery(gachaEnhancedStatsQueryOptions({ topN: 20, includeTiming: true }));
    const server = useGamedataServer();
    const operators = useQuery(operatorsIndexQueryOptions(server));
    // Community pulls are every account's, all imported from the Global client: banners come from
    // that table, not the reader's text pick, or no pool id would resolve.
    const banners = useQuery(bannersQueryOptions(PULLS_GAMEDATA_SERVER));
    const perBannerStats = useQuery(perBannerStatsQueryOptions());
    const { isAuthenticated } = useAuth();
    const myStats = useQuery(myGachaStatsQueryOptions(isAuthenticated));

    const operatorsById = useMemo(() => {
        const map = new Map<string, IOperatorIndexEntry>();
        for (const entry of operators.data ?? []) map.set(entry.id, entry);
        return map;
    }, [operators.data]);

    // Oldest first: the timing chart wants chronological order, and the runs
    // list re-sorts by status but needs a stable input.
    const bannersSorted = useMemo<IBanner[]>(() => {
        const list = banners.data ?? [];
        return [...list].sort((a, b) => a.openTime - b.openTime);
    }, [banners.data]);

    const perBannerById = useMemo(() => {
        const map = new Map<string, IBannerPullStat>();
        for (const s of perBannerStats.data ?? []) map.set(s.poolId, s);
        return map;
    }, [perBannerStats.data]);

    const personalRates = useMemo<IPersonalRarityRates | null>(() => {
        const s = myStats.data;
        if (!s || !s.total_pulls || s.total_pulls <= 0) return null;
        const six = s.six_star_count ?? 0;
        const five = s.five_star_count ?? 0;
        const four = s.four_star_count ?? 0;
        // v_gacha_stats has no 3★ count, so it comes off the total. Anything
        // outside 3-6 (none in current data) folds in here until the view is widened.
        const three = Math.max(0, s.total_pulls - six - five - four);
        return {
            totalPulls: s.total_pulls,
            totalSixStars: six,
            totalFiveStars: five,
            totalFourStars: four,
            totalThreeStars: three,
        };
    }, [myStats.data]);

    const data = enhanced.data ?? null;
    const isLoading = enhanced.isLoading;

    return (
        <>
            <div className={styles.pageAmbient} aria-hidden />
            <section className="page-shell flex flex-col gap-6 [--page-max:1320px]">
                <PageHeader data={data} isLoading={isLoading} />

                {/* The KPIs, leaderboard, rarity and timing panels all read this one query; on a failure
                    they would each sit on their placeholders, so one failure line stands in for all four. */}
                {enhanced.isError && !data ? (
                    <LoadFailed message={t("community.error.load")} onRetry={() => void enhanced.refetch()} />
                ) : (
                    <>
                        <KpiStrip data={data} />

                        <div className="grid grid-cols-1 gap-4 lg:grid-cols-[1.3fr_1fr]">
                            <div className="lg:relative lg:min-h-0">
                                <Leaderboard ops={data?.mostCommonOperators ?? []} operatorsById={operatorsById} isLoading={isLoading} />
                            </div>
                            <RarityPanel data={data} personal={personalRates} />
                        </div>

                        <TimingPanel timing={data?.pullTiming} firstPullAt={data?.collectiveStats.firstPullAt} banners={bannersSorted} />
                    </>
                )}

                {/* The banner list and its per-banner stats are separate queries: without the list the runs
                    panel is empty, and without the stats its rows show no pull counts. */}
                {banners.isError ? <LoadFailed message={t("community.error.banners")} onRetry={() => void banners.refetch()} /> : perBannerStats.isError ? <LoadFailed message={t("community.error.bannerStats")} onRetry={() => void perBannerStats.refetch()} /> : null}

                <BannerRunsPanel banners={bannersSorted} operatorsById={operatorsById} statsById={perBannerById} isLoading={banners.isLoading} />
            </section>
        </>
    );
}
