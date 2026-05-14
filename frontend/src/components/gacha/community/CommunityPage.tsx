import { useQuery } from "@tanstack/react-query";
import { useMemo } from "react";
import { useAuth } from "#/hooks/use-auth";
import { bannersQueryOptions, gachaEnhancedStatsQueryOptions, type IBanner, type IBannerPullStat, myGachaStatsQueryOptions, perBannerStatsQueryOptions } from "#/lib/api/gacha";
import { operatorsIndexQueryOptions } from "#/lib/api/operators";
import type { IOperatorIndexEntry } from "#/types/operators";
import { BannerRunsPanel } from "./impl/BannerRunsPanel";
import styles from "./impl/CommunityPage.module.css";
import { KpiStrip } from "./impl/KpiStrip";
import { Leaderboard } from "./impl/Leaderboard";
import { PageHeader } from "./impl/PageHeader";
import { type IPersonalRarityRates, RarityPanel } from "./impl/RarityPanel";
import { TimingPanel } from "./impl/TimingPanel";

export function CommunityPage() {
    const enhanced = useQuery(gachaEnhancedStatsQueryOptions({ topN: 20, includeTiming: true }));
    const operators = useQuery(operatorsIndexQueryOptions());
    const banners = useQuery(bannersQueryOptions());
    const perBannerStats = useQuery(perBannerStatsQueryOptions());
    const { isAuthenticated } = useAuth();
    const myStats = useQuery(myGachaStatsQueryOptions(isAuthenticated));

    const operatorsById = useMemo(() => {
        const map = new Map<string, IOperatorIndexEntry>();
        for (const entry of operators.data ?? []) map.set(entry.id, entry);
        return map;
    }, [operators.data]);

    // Banners sorted oldest-first by openTime. Useful for the timing-chart
    // overlay (chronological) and the banner-runs list (which sorts by status
    // and recency itself but takes a stable input).
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
        // The v_gacha_stats view doesn't return a 3★ count, so derive it from the
        // total. Anything outside the 3-6 range (shouldn't exist in current data)
        // would fold into this bucket - that's acceptable until the view is widened.
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
            <section className="mx-auto flex w-full max-w-330 flex-col gap-6 px-8 pb-15 pt-7 max-[760px]:px-4 max-[760px]:pt-5 max-[760px]:pb-10">
                <PageHeader data={data} isLoading={isLoading} />

                {enhanced.isError ? (
                    <div className="rounded-[14px] border border-destructive/30 bg-destructive/8 px-5 py-4 font-sans text-sm text-foreground/90">
                        <strong className="font-semibold text-foreground">Couldn&rsquo;t load community stats.</strong> {(enhanced.error as Error)?.message ?? "Unknown error."}
                    </div>
                ) : null}

                <KpiStrip data={data} />

                <div className="grid gap-4 grid-cols-1 lg:grid-cols-[1.3fr_1fr]">
                    <Leaderboard ops={data?.mostCommonOperators ?? []} operatorsById={operatorsById} isLoading={isLoading} />
                    <RarityPanel data={data} personal={personalRates} />
                </div>

                <TimingPanel timing={data?.pullTiming} firstPullAt={data?.collectiveStats.firstPullAt} banners={bannersSorted} />

                <BannerRunsPanel banners={bannersSorted} operatorsById={operatorsById} statsById={perBannerById} isLoading={banners.isLoading} />
            </section>
        </>
    );
}
