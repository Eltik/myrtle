import { BarChart3, Sparkles, Star, Users } from "lucide-react";
import type { GetServerSideProps, NextPage } from "next";
import { BannerActivityChart, BannerExplorer, calculateDerivedData, DataSourceNotice, getLuckStatus, MostCommonOperators, PityStatistics, PullActivityChart, PullRateAnalysis, PullTimingCharts, RarityDistribution, StatCard, StatsHeader } from "~/components/gacha/community";
import { buildBannerOverlays } from "~/components/gacha/community/impl/banner-helpers";
import { SEO } from "~/components/seo";
import { formatRate, RARITY_TIER_MAP } from "~/lib/gacha-utils";
import type { GachaEnhancedStats, GachaPoolClient, GachaPoolsResponse } from "~/types/api";

interface GlobalGachaStatsPageProps {
    stats: GachaEnhancedStats | null;
    pools: GachaPoolClient[];
    error?: string;
}

const GlobalGachaStatsPage: NextPage<GlobalGachaStatsPageProps> = ({ stats, pools, error }) => {
    // Error state
    if (error || !stats) {
        return (
            <>
                <SEO description="Community-wide Arknights gacha statistics and pull rates." path="/gacha/community" title="Community Stats" />
                <div className="container mx-auto flex min-h-[50vh] items-center justify-center p-4">
                    <div className="text-center">
                        <h1 className="mb-4 font-bold text-4xl">Statistics Unavailable</h1>
                        <p className="text-muted-foreground">{error ?? "Unable to load global statistics at this time."}</p>
                    </div>
                </div>
            </>
        );
    }

    // Calculate all derived data
    const { actualRates, luckScore, rateComparisonData, operatorsByRarity, hourlyData, dailyData, dateData, rarityData } = calculateDerivedData(stats);
    const luckStatus = getLuckStatus(luckScore);
    const bannerOverlays = buildBannerOverlays(pools, dateData);

    return (
        <>
            <SEO description={`Community gacha statistics from ${stats.collectiveStats.totalUsers.toLocaleString()} players and ${stats.collectiveStats.totalPulls.toLocaleString()} pulls.`} keywords={["gacha statistics", "pull rates", "6-star rate", "Arknights gacha"]} path="/gacha/community" title="Community Stats" />
            <div className="container mx-auto space-y-8 p-4 py-8">
                {/* Header */}
                <StatsHeader cached={stats.cached} computedAt={stats.computedAt} totalUsers={stats.collectiveStats.totalUsers} />

                {/* Main Statistics Cards */}
                <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
                    <StatCard delay={0.1} description="Across all participating users" icon={BarChart3} title="Total Pulls" value={stats.collectiveStats.totalPulls} />
                    <StatCard delay={0.2} description="Anonymous data contributors" icon={Users} title="Contributing Players" value={stats.collectiveStats.totalUsers} />
                    <StatCard delay={0.3} description={`${formatRate(actualRates[6])} pull rate`} icon={Sparkles} iconClassName="text-orange-500" title="6-Star Operators" value={stats.collectiveStats.totalSixStars} valueClassName="text-orange-500" />
                    <StatCard delay={0.4} description={`${formatRate(actualRates[5])} pull rate`} icon={Star} iconClassName="text-yellow-500" title="5-Star Operators" value={stats.collectiveStats.totalFiveStars} valueClassName="text-yellow-500" />
                </div>

                {/* Pull Rate Analysis */}
                <PullRateAnalysis actualRates={actualRates} averagePullsToFiveStar={stats.averagePullsToFiveStar} averagePullsToSixStar={stats.averagePullsToSixStar} />

                {/* Pity Statistics */}
                <PityStatistics averagePullsToFiveStar={stats.averagePullsToFiveStar} averagePullsToSixStar={stats.averagePullsToSixStar} />

                {/* Rarity Distribution */}
                <RarityDistribution collectiveStats={stats.collectiveStats} luckStatus={luckStatus} rarityData={rarityData} rateComparisonData={rateComparisonData} />

                {/* Pull Timing Charts */}
                {stats.pullTiming && <PullTimingCharts dailyData={dailyData} hourlyData={hourlyData} />}

                {/* Pull Activity Over Time */}
                {pools.length > 0 && bannerOverlays.length > 0 ? <BannerActivityChart bannerOverlays={bannerOverlays} dateData={dateData} /> : <PullActivityChart dateData={dateData} />}

                {/* Banner Explorer */}
                {pools.length > 0 && <BannerExplorer byDate={stats.pullTiming?.byDate} pools={pools} />}

                {/* Most Common Operators */}
                <MostCommonOperators operatorsByRarity={operatorsByRarity} />

                {/* Data Source Notice */}
                <DataSourceNotice />
            </div>
        </>
    );
};

export const getServerSideProps: GetServerSideProps<GlobalGachaStatsPageProps> = async () => {
    try {
        const { env } = await import("~/env");

        const headers = {
            "Content-Type": "application/json",
            "X-Internal-Service-Key": env.INTERNAL_SERVICE_KEY,
        };

        // Fetch enhanced stats, operator data, and pool data in parallel
        const backendURL = new URL("/gacha/stats/enhanced", env.BACKEND_URL);
        backendURL.searchParams.set("top_n", "20");
        backendURL.searchParams.set("include_timing", "true");

        const operatorsURL = new URL("/static/operators", env.BACKEND_URL);
        operatorsURL.searchParams.set("fields", "id,name,rarity,profession");
        operatorsURL.searchParams.set("limit", "1000");

        const poolsURL = new URL("/static/gacha/pools", env.BACKEND_URL);

        const [statsResponse, operatorsResponse, poolsResponse] = await Promise.all([fetch(backendURL.toString(), { method: "GET", headers }), fetch(operatorsURL.toString(), { method: "GET", headers }).catch(() => null), fetch(poolsURL.toString(), { method: "GET", headers }).catch(() => null)]);

        if (!statsResponse.ok) {
            console.error(`Enhanced stats fetch failed: ${statsResponse.status}`);
            return {
                props: {
                    stats: null,
                    pools: [],
                    error: "Failed to fetch statistics",
                },
            };
        }

        const stats: GachaEnhancedStats = await statsResponse.json();

        // Enrich operator data with correct rarity/name from static game data
        if (operatorsResponse?.ok) {
            try {
                const operatorData: { operators?: Array<{ id: string | null; name: string; rarity: string; profession: string }> } = await operatorsResponse.json();
                if (operatorData.operators) {
                    const lookup = new Map<string, { name: string; rarity: number }>();
                    for (const op of operatorData.operators) {
                        if (!op.id) continue;
                        lookup.set(op.id, {
                            name: op.name,
                            rarity: RARITY_TIER_MAP[op.rarity] ?? 3,
                        });
                    }

                    stats.mostCommonOperators = stats.mostCommonOperators.map((op) => {
                        const staticOp = lookup.get(op.charId);
                        if (staticOp) {
                            return { ...op, rarity: staticOp.rarity, charName: staticOp.name };
                        }
                        return op;
                    });
                }
            } catch (err) {
                console.error("Failed to parse operator data for enrichment:", err);
            }
        }

        // Parse pool data
        let pools: GachaPoolClient[] = [];
        if (poolsResponse?.ok) {
            try {
                const poolsData: GachaPoolsResponse = await poolsResponse.json();
                pools = poolsData.pools ?? [];
            } catch (err) {
                console.error("Failed to parse pool data:", err);
            }
        }

        return {
            props: {
                stats,
                pools,
            },
        };
    } catch (error) {
        console.error("Error fetching global gacha stats:", error);
        return {
            props: {
                stats: null,
                pools: [],
                error: error instanceof Error ? error.message : "An unexpected error occurred",
            },
        };
    }
};

export default GlobalGachaStatsPage;
