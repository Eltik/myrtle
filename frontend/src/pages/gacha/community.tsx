import { BarChart3, Sparkles, Star, Users } from "lucide-react";
import type { GetServerSideProps, NextPage } from "next";
import { calculateDerivedData, DataSourceNotice, getLuckStatus, MostCommonOperators, PityStatistics, PullActivityChart, PullRateAnalysis, PullTimingCharts, RarityDistribution, StatCard, StatsHeader } from "~/components/gacha/community";
import { SEO } from "~/components/seo";
import { formatRate } from "~/lib/gacha-utils";
import type { GachaEnhancedStats } from "~/types/api";

interface GlobalGachaStatsPageProps {
    stats: GachaEnhancedStats | null;
    error?: string;
}

const GlobalGachaStatsPage: NextPage<GlobalGachaStatsPageProps> = ({ stats, error }) => {
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
                <PullActivityChart dateData={dateData} />

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

        // Fetch enhanced stats with timing data and top 20 operators
        const backendURL = new URL("/gacha/stats/enhanced", env.BACKEND_URL);
        backendURL.searchParams.set("top_n", "20");
        backendURL.searchParams.set("include_timing", "true");

        const response = await fetch(backendURL.toString(), {
            method: "GET",
            headers: {
                "Content-Type": "application/json",
                "X-Internal-Service-Key": env.INTERNAL_SERVICE_KEY,
            },
        });

        if (!response.ok) {
            console.error(`Enhanced stats fetch failed: ${response.status}`);
            return {
                props: {
                    stats: null,
                    error: "Failed to fetch statistics",
                },
            };
        }

        const stats: GachaEnhancedStats = await response.json();

        return {
            props: {
                stats,
            },
        };
    } catch (error) {
        console.error("Error fetching global gacha stats:", error);
        return {
            props: {
                stats: null,
                error: error instanceof Error ? error.message : "An unexpected error occurred",
            },
        };
    }
};

export default GlobalGachaStatsPage;
