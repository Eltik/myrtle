import { BarChart3, Calendar, Clock, Dices, Minus, PieChartIcon, Sparkles, Star, Target, TrendingDown, TrendingUp, Users } from "lucide-react";
import type { GetServerSideProps, NextPage } from "next";
import Image from "next/image";
import { Bar, BarChart, Cell, Line, LineChart, Pie, PieChart, ResponsiveContainer, XAxis, YAxis } from "recharts";
import { SEO } from "~/components/seo";
import { AnimatedNumber } from "~/components/ui/motion-primitives/animated-number";
import { InView } from "~/components/ui/motion-primitives/in-view";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "~/components/ui/shadcn/card";
import { ChartContainer, ChartLegend, ChartLegendContent, ChartTooltip, ChartTooltipContent } from "~/components/ui/shadcn/chart";
import { Separator } from "~/components/ui/shadcn/separator";
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "~/components/ui/shadcn/tooltip";
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
                <SEO description="Community-wide Arknights gacha statistics and pull rates." path="/stats/gacha" title="Global Gacha Statistics" />
                <div className="container mx-auto flex min-h-[50vh] items-center justify-center p-4">
                    <div className="text-center">
                        <h1 className="mb-4 font-bold text-4xl">Statistics Unavailable</h1>
                        <p className="text-muted-foreground">{error ?? "Unable to load global statistics at this time."}</p>
                    </div>
                </div>
            </>
        );
    }

    // Expected rates for comparison (official gacha rates)
    const expectedRates = {
        6: 0.02, // 2%
        5: 0.08, // 8%
        4: 0.5, // 50%
        3: 0.4, // 40%
    };

    // Calculate actual rates from totals
    const totalPulls = stats.collectiveStats.totalPulls;
    const actualRates = {
        6: totalPulls > 0 ? stats.collectiveStats.totalSixStars / totalPulls : 0,
        5: totalPulls > 0 ? stats.collectiveStats.totalFiveStars / totalPulls : 0,
        4: totalPulls > 0 ? stats.collectiveStats.totalFourStars / totalPulls : 0,
        3: totalPulls > 0 ? stats.collectiveStats.totalThreeStars / totalPulls : 0,
    };

    // Calculate overall luck score (weighted average of rate differences)
    // Positive = lucky, Negative = unlucky
    const luckScore =
        ((actualRates[6] - expectedRates[6]) / expectedRates[6]) * 0.5 + // 6★ weighted heavily
        ((actualRates[5] - expectedRates[5]) / expectedRates[5]) * 0.3 + // 5★ weighted moderately
        ((actualRates[4] - expectedRates[4]) / expectedRates[4]) * 0.1 + // 4★ weighted lightly
        ((actualRates[3] - expectedRates[3]) / expectedRates[3]) * 0.1; // 3★ weighted lightly

    // Determine luck status
    const getLuckStatus = () => {
        if (luckScore > 0.05) return { label: "Very Lucky", color: "text-green-500", bg: "bg-green-500/10", icon: TrendingUp };
        if (luckScore > 0.01) return { label: "Lucky", color: "text-green-400", bg: "bg-green-400/10", icon: TrendingUp };
        if (luckScore > -0.01) return { label: "Average", color: "text-muted-foreground", bg: "bg-muted", icon: Minus };
        if (luckScore > -0.05) return { label: "Unlucky", color: "text-yellow-500", bg: "bg-yellow-500/10", icon: TrendingDown };
        return { label: "Very Unlucky", color: "text-red-500", bg: "bg-red-500/10", icon: TrendingDown };
    };
    const luckStatus = getLuckStatus();

    // Rate comparison data for progress bars
    const rateComparisonData = [
        { rarity: 6, label: "6★", actual: actualRates[6], expected: expectedRates[6], color: "#f97316", bgColor: "bg-orange-500" },
        { rarity: 5, label: "5★", actual: actualRates[5], expected: expectedRates[5], color: "#eab308", bgColor: "bg-yellow-500" },
        { rarity: 4, label: "4★", actual: actualRates[4], expected: expectedRates[4], color: "#a855f7", bgColor: "bg-purple-500" },
        { rarity: 3, label: "3★", actual: actualRates[3], expected: expectedRates[3], color: "#3b82f6", bgColor: "bg-blue-500" },
    ];

    // Group operators by rarity for display
    const operatorsByRarity = {
        6: stats.mostCommonOperators.filter((op) => op.rarity === 6),
        5: stats.mostCommonOperators.filter((op) => op.rarity === 5),
        4: stats.mostCommonOperators.filter((op) => op.rarity === 4),
        3: stats.mostCommonOperators.filter((op) => op.rarity === 3),
    };

    // Day names for chart display
    const dayNames = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];

    // Chart data for pull timing
    const hourlyData =
        stats.pullTiming?.byHour.map((item) => ({
            hour: `${item.hour}:00`,
            pulls: item.pullCount,
            percentage: item.percentage,
        })) ?? [];

    const dailyData =
        stats.pullTiming?.byDayOfWeek.map((item) => ({
            day: dayNames[item.day] ?? item.dayName,
            pulls: item.pullCount,
            percentage: item.percentage,
        })) ?? [];

    // Time series data for pulls by date (all historical data)
    const dateData =
        stats.pullTiming?.byDate?.map((item) => ({
            date: new Date(item.date).toLocaleDateString(undefined, { month: "short", day: "numeric" }),
            fullDate: item.date,
            pulls: item.pullCount,
        })) ?? [];

    // Rarity distribution for pie chart with explicit colors
    const rarityData = [
        { name: "6-Star", value: stats.collectiveStats.totalSixStars, color: "#f97316" }, // orange-500
        { name: "5-Star", value: stats.collectiveStats.totalFiveStars, color: "#eab308" }, // yellow-500
        { name: "4-Star", value: stats.collectiveStats.totalFourStars, color: "#a855f7" }, // purple-500
        { name: "3-Star", value: stats.collectiveStats.totalThreeStars, color: "#3b82f6" }, // blue-500
    ];

    return (
        <>
            <SEO
                description={`Community gacha statistics from ${stats.collectiveStats.totalUsers.toLocaleString()} players and ${stats.collectiveStats.totalPulls.toLocaleString()} pulls.`}
                keywords={["gacha statistics", "pull rates", "6-star rate", "Arknights gacha"]}
                path="/stats/gacha"
                title="Global Gacha Statistics"
            />
            <div className="container mx-auto space-y-8 p-4 py-8">
                {/* Header */}
                <InView
                    once
                    transition={{ duration: 0.6, ease: "easeOut" }}
                    variants={{
                        hidden: { opacity: 0, y: 20 },
                        visible: { opacity: 1, y: 0 },
                    }}
                >
                    <div className="space-y-2 text-center">
                        <h1 className="text-balance font-bold text-4xl md:text-5xl">Global Gacha Statistics</h1>
                        <p className="mx-auto max-w-2xl text-balance text-lg text-muted-foreground">Community-wide pull data from {stats.collectiveStats.totalUsers.toLocaleString()} players sharing their statistics anonymously.</p>
                        <p className="text-muted-foreground text-sm">
                            Last updated:{" "}
                            {new Date(stats.computedAt).toLocaleString(undefined, {
                                year: "numeric",
                                month: "short",
                                day: "numeric",
                                hour: "2-digit",
                                minute: "2-digit",
                            })}{" "}
                            {stats.cached && "(Cached)"}
                        </p>
                    </div>
                </InView>

                {/* Main Statistics Cards */}
                <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
                    <InView
                        once
                        transition={{ duration: 0.5, ease: "easeOut", delay: 0.1 }}
                        variants={{
                            hidden: { opacity: 0, scale: 0.9 },
                            visible: { opacity: 1, scale: 1 },
                        }}
                    >
                        <Card>
                            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                                <CardTitle className="font-medium text-sm">Total Pulls</CardTitle>
                                <BarChart3 className="h-4 w-4 text-muted-foreground" />
                            </CardHeader>
                            <CardContent>
                                <div className="font-bold text-2xl">
                                    <AnimatedNumber springOptions={{ bounce: 0, duration: 2000 }} value={stats.collectiveStats.totalPulls} />
                                </div>
                                <p className="text-muted-foreground text-xs">Across all participating users</p>
                            </CardContent>
                        </Card>
                    </InView>

                    <InView
                        once
                        transition={{ duration: 0.5, ease: "easeOut", delay: 0.2 }}
                        variants={{
                            hidden: { opacity: 0, scale: 0.9 },
                            visible: { opacity: 1, scale: 1 },
                        }}
                    >
                        <Card>
                            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                                <CardTitle className="font-medium text-sm">Contributing Players</CardTitle>
                                <Users className="h-4 w-4 text-muted-foreground" />
                            </CardHeader>
                            <CardContent>
                                <div className="font-bold text-2xl">
                                    <AnimatedNumber springOptions={{ bounce: 0, duration: 2000 }} value={stats.collectiveStats.totalUsers} />
                                </div>
                                <p className="text-muted-foreground text-xs">Anonymous data contributors</p>
                            </CardContent>
                        </Card>
                    </InView>

                    <InView
                        once
                        transition={{ duration: 0.5, ease: "easeOut", delay: 0.3 }}
                        variants={{
                            hidden: { opacity: 0, scale: 0.9 },
                            visible: { opacity: 1, scale: 1 },
                        }}
                    >
                        <Card>
                            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                                <CardTitle className="font-medium text-sm">6-Star Operators</CardTitle>
                                <Sparkles className="h-4 w-4 text-orange-500" />
                            </CardHeader>
                            <CardContent>
                                <div className="font-bold text-2xl text-orange-500">
                                    <AnimatedNumber springOptions={{ bounce: 0, duration: 2000 }} value={stats.collectiveStats.totalSixStars} />
                                </div>
                                <p className="text-muted-foreground text-xs">{formatRate(stats.pullRates.sixStarRate)} pull rate</p>
                            </CardContent>
                        </Card>
                    </InView>

                    <InView
                        once
                        transition={{ duration: 0.5, ease: "easeOut", delay: 0.4 }}
                        variants={{
                            hidden: { opacity: 0, scale: 0.9 },
                            visible: { opacity: 1, scale: 1 },
                        }}
                    >
                        <Card>
                            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                                <CardTitle className="font-medium text-sm">5-Star Operators</CardTitle>
                                <Star className="h-4 w-4 text-yellow-500" />
                            </CardHeader>
                            <CardContent>
                                <div className="font-bold text-2xl text-yellow-500">
                                    <AnimatedNumber springOptions={{ bounce: 0, duration: 2000 }} value={stats.collectiveStats.totalFiveStars} />
                                </div>
                                <p className="text-muted-foreground text-xs">{formatRate(stats.pullRates.fiveStarRate)} pull rate</p>
                            </CardContent>
                        </Card>
                    </InView>
                </div>

                {/* Pull Rate Analysis */}
                <InView
                    once
                    transition={{ duration: 0.5, ease: "easeOut" }}
                    variants={{
                        hidden: { opacity: 0, y: 20 },
                        visible: { opacity: 1, y: 0 },
                    }}
                >
                    <Card>
                        <CardHeader>
                            <CardTitle className="flex items-center gap-2">
                                <TrendingUp className="h-5 w-5" />
                                Pull Rate Analysis
                            </CardTitle>
                            <CardDescription>Community average rates compared to expected probabilities</CardDescription>
                        </CardHeader>
                        <CardContent className="space-y-6">
                            {/* 6-Star Rate Comparison */}
                            <div className="space-y-2">
                                <div className="flex items-center justify-between">
                                    <span className="font-semibold">6-Star Rate</span>
                                    <div className="flex items-center gap-2">
                                        <span className={`font-bold ${stats.pullRates.sixStarRate >= expectedRates[6] ? "text-green-500" : "text-yellow-500"}`}>{formatRate(stats.pullRates.sixStarRate)}</span>
                                        <span className="text-muted-foreground text-sm">(Expected: {formatRate(expectedRates[6])})</span>
                                    </div>
                                </div>
                                <div className="relative h-4 overflow-hidden rounded-full bg-secondary">
                                    <div
                                        className="h-full rounded-full bg-linear-to-r from-orange-500 to-orange-400 transition-all duration-1000"
                                        style={{
                                            width: `${Math.min((stats.pullRates.sixStarRate / expectedRates[6]) * 100, 100)}%`,
                                        }}
                                    />
                                </div>
                                <p className="text-muted-foreground text-xs">
                                    Community rate is {Math.abs((stats.pullRates.sixStarRate - expectedRates[6]) * 100).toFixed(2)}% {stats.pullRates.sixStarRate >= expectedRates[6] ? "above" : "below"} expected
                                </p>
                            </div>

                            <Separator />

                            {/* 5-Star Rate Comparison */}
                            <div className="space-y-2">
                                <div className="flex items-center justify-between">
                                    <span className="font-semibold">5-Star Rate</span>
                                    <div className="flex items-center gap-2">
                                        <span className={`font-bold ${stats.pullRates.fiveStarRate >= expectedRates[5] ? "text-green-500" : "text-yellow-500"}`}>{formatRate(stats.pullRates.fiveStarRate)}</span>
                                        <span className="text-muted-foreground text-sm">(Expected: {formatRate(expectedRates[5])})</span>
                                    </div>
                                </div>
                                <div className="relative h-4 overflow-hidden rounded-full bg-secondary">
                                    <div
                                        className="h-full rounded-full bg-linear-to-r from-yellow-500 to-yellow-400 transition-all duration-1000"
                                        style={{
                                            width: `${Math.min((stats.pullRates.fiveStarRate / expectedRates[5]) * 100, 100)}%`,
                                        }}
                                    />
                                </div>
                                <p className="text-muted-foreground text-xs">
                                    Community rate is {Math.abs((stats.pullRates.fiveStarRate - expectedRates[5]) * 100).toFixed(2)}% {stats.pullRates.fiveStarRate >= expectedRates[5] ? "above" : "below"} expected
                                </p>
                            </div>

                            <Separator />

                            {/* Average Pulls */}
                            <div className="grid gap-4 sm:grid-cols-2">
                                <div className="space-y-1 text-center">
                                    <p className="text-muted-foreground text-sm">Avg Pulls per 6★</p>
                                    <p className="font-bold text-3xl text-orange-500">
                                        <AnimatedNumber decimals={1} springOptions={{ bounce: 0, duration: 2000 }} value={stats.averagePullsToSixStar} />
                                    </p>
                                </div>
                                <div className="space-y-1 text-center">
                                    <p className="text-muted-foreground text-sm">Avg Pulls per 5★</p>
                                    <p className="font-bold text-3xl text-yellow-500">
                                        <AnimatedNumber decimals={1} springOptions={{ bounce: 0, duration: 2000 }} value={stats.averagePullsToFiveStar} />
                                    </p>
                                </div>
                            </div>
                        </CardContent>
                    </Card>
                </InView>

                {/* Pity Statistics */}
                <InView
                    once
                    transition={{ duration: 0.5, ease: "easeOut" }}
                    variants={{
                        hidden: { opacity: 0, y: 20 },
                        visible: { opacity: 1, y: 0 },
                    }}
                >
                    <Card>
                        <CardHeader>
                            <CardTitle className="flex items-center gap-2">
                                <Target className="h-5 w-5" />
                                Average Pity Statistics
                            </CardTitle>
                            <CardDescription>Community average pulls required to obtain rare operators</CardDescription>
                        </CardHeader>
                        <CardContent>
                            <div className="grid gap-6 md:grid-cols-2">
                                {/* 6-Star Pity */}
                                <div className="space-y-4">
                                    <div className="flex items-center gap-3">
                                        <div className="flex h-12 w-12 items-center justify-center rounded-lg bg-orange-500/10">
                                            <Dices className="h-6 w-6 text-orange-500" />
                                        </div>
                                        <div>
                                            <p className="font-semibold text-muted-foreground text-sm">6-Star Pity</p>
                                            <p className="font-bold text-2xl text-orange-500">
                                                <AnimatedNumber decimals={1} springOptions={{ bounce: 0, duration: 2000 }} value={stats.averagePullsToSixStar} />
                                                <span className="ml-1 text-base">pulls</span>
                                            </p>
                                        </div>
                                    </div>
                                    <div className="space-y-2 rounded-lg bg-orange-500/5 p-4">
                                        <div className="flex justify-between text-sm">
                                            <span className="text-muted-foreground">Expected pity:</span>
                                            <span className="font-medium">50 pulls (guaranteed at 99)</span>
                                        </div>
                                        <div className="flex justify-between text-sm">
                                            <span className="text-muted-foreground">Community average:</span>
                                            <span className="font-semibold text-orange-500">{stats.averagePullsToSixStar.toFixed(1)} pulls</span>
                                        </div>
                                        <Separator className="my-2" />
                                        <p className="text-muted-foreground text-xs">{stats.averagePullsToSixStar < 50 ? `Community pulls ${(50 - stats.averagePullsToSixStar).toFixed(1)} pulls better than expected!` : `Community pulls ${(stats.averagePullsToSixStar - 50).toFixed(1)} pulls more than expected.`}</p>
                                    </div>
                                </div>

                                {/* 5-Star Pity */}
                                <div className="space-y-4">
                                    <div className="flex items-center gap-3">
                                        <div className="flex h-12 w-12 items-center justify-center rounded-lg bg-yellow-500/10">
                                            <Sparkles className="h-6 w-6 text-yellow-500" />
                                        </div>
                                        <div>
                                            <p className="font-semibold text-muted-foreground text-sm">5-Star Pity</p>
                                            <p className="font-bold text-2xl text-yellow-500">
                                                <AnimatedNumber decimals={1} springOptions={{ bounce: 0, duration: 2000 }} value={stats.averagePullsToFiveStar} />
                                                <span className="ml-1 text-base">pulls</span>
                                            </p>
                                        </div>
                                    </div>
                                    <div className="space-y-2 rounded-lg bg-yellow-500/5 p-4">
                                        <div className="flex justify-between text-sm">
                                            <span className="text-muted-foreground">Expected rate:</span>
                                            <span className="font-medium">1 per 12.5 pulls (8%)</span>
                                        </div>
                                        <div className="flex justify-between text-sm">
                                            <span className="text-muted-foreground">Community average:</span>
                                            <span className="font-semibold text-yellow-500">{stats.averagePullsToFiveStar.toFixed(1)} pulls</span>
                                        </div>
                                        <Separator className="my-2" />
                                        <p className="text-muted-foreground text-xs">
                                            {stats.averagePullsToFiveStar < 12.5 ? `Community pulls ${(12.5 - stats.averagePullsToFiveStar).toFixed(1)} pulls better than expected!` : `Community pulls ${(stats.averagePullsToFiveStar - 12.5).toFixed(1)} pulls more than expected.`}
                                        </p>
                                    </div>
                                </div>
                            </div>
                        </CardContent>
                    </Card>
                </InView>

                {/* Rarity Distribution Pie Chart */}
                <InView
                    once
                    transition={{ duration: 0.5, ease: "easeOut" }}
                    variants={{
                        hidden: { opacity: 0, y: 20 },
                        visible: { opacity: 1, y: 0 },
                    }}
                >
                    <Card>
                        <CardHeader>
                            <div className="flex items-center justify-between">
                                <div>
                                    <CardTitle className="flex items-center gap-2">
                                        <PieChartIcon className="h-5 w-5" />
                                        Rarity Distribution
                                    </CardTitle>
                                    <CardDescription>Breakdown of pulled operators by rarity</CardDescription>
                                </div>
                                {/* Luck Indicator Badge */}
                                <div className={`flex items-center gap-2 rounded-full px-3 py-1.5 ${luckStatus.bg}`}>
                                    <luckStatus.icon className={`h-4 w-4 ${luckStatus.color}`} />
                                    <span className={`font-semibold text-sm ${luckStatus.color}`}>{luckStatus.label}</span>
                                </div>
                            </div>
                        </CardHeader>
                        <CardContent>
                            <div className="grid gap-6 lg:grid-cols-2">
                                {/* Pie Chart */}
                                <div>
                                    <ChartContainer
                                        className="mx-auto aspect-square h-64"
                                        config={{
                                            sixStar: {
                                                label: "6-Star",
                                                color: "#f97316",
                                            },
                                            fiveStar: {
                                                label: "5-Star",
                                                color: "#eab308",
                                            },
                                            fourStar: {
                                                label: "4-Star",
                                                color: "#a855f7",
                                            },
                                            threeStar: {
                                                label: "3-Star",
                                                color: "#3b82f6",
                                            },
                                        }}
                                    >
                                        <ResponsiveContainer height="100%" width="100%">
                                            <PieChart>
                                                <ChartTooltip content={<ChartTooltipContent hideLabel />} cursor={false} />
                                                <Pie cx="50%" cy="50%" data={rarityData} dataKey="value" innerRadius={60} nameKey="name" outerRadius={80} paddingAngle={2}>
                                                    {rarityData.map((entry) => (
                                                        <Cell fill={entry.color} key={`cell-${entry.name}`} />
                                                    ))}
                                                </Pie>
                                                <ChartLegend content={<ChartLegendContent />} verticalAlign="bottom" />
                                            </PieChart>
                                        </ResponsiveContainer>
                                    </ChartContainer>
                                </div>

                                {/* Rate Comparison Bars */}
                                <div className="space-y-4">
                                    <h4 className="font-semibold text-sm">Actual vs Expected Rates</h4>
                                    {rateComparisonData.map((item) => {
                                        const actualPercent = item.actual * 100;
                                        const expectedPercent = item.expected * 100;
                                        const ratio = item.expected > 0 ? item.actual / item.expected : 0;
                                        const isLucky = ratio >= 1;
                                        const barWidth = Math.min(ratio * 100, 150); // Cap at 150% for visual

                                        return (
                                            <div className="space-y-1.5" key={item.rarity}>
                                                <div className="flex items-center justify-between text-sm">
                                                    <span className="font-medium" style={{ color: item.color }}>
                                                        {item.label}
                                                    </span>
                                                    <div className="flex items-center gap-2">
                                                        <span className={isLucky ? "text-green-500" : "text-yellow-500"}>{actualPercent.toFixed(2)}%</span>
                                                        <span className="text-muted-foreground">/ {expectedPercent.toFixed(0)}%</span>
                                                    </div>
                                                </div>
                                                <div className="relative h-2 overflow-hidden rounded-full bg-secondary">
                                                    {/* Expected rate marker */}
                                                    <div className="absolute top-0 h-full w-0.5 bg-muted-foreground/50" style={{ left: "66.67%" }} />
                                                    {/* Actual rate bar */}
                                                    <div className={`h-full rounded-full transition-all duration-1000 ${item.bgColor}`} style={{ width: `${barWidth * 0.6667}%` }} />
                                                </div>
                                                <p className="text-muted-foreground text-xs">{isLucky ? <span className="text-green-500">+{((ratio - 1) * 100).toFixed(1)}% above expected</span> : <span className="text-yellow-500">{((1 - ratio) * 100).toFixed(1)}% below expected</span>}</p>
                                            </div>
                                        );
                                    })}
                                </div>
                            </div>

                            <Separator className="my-6" />

                            {/* Totals Grid */}
                            <div className="grid grid-cols-4 gap-4 text-center">
                                <div>
                                    <p className="text-muted-foreground text-sm">6★ Total</p>
                                    <p className="font-bold text-orange-500 text-xl">{stats.collectiveStats.totalSixStars.toLocaleString()}</p>
                                </div>
                                <div>
                                    <p className="text-muted-foreground text-sm">5★ Total</p>
                                    <p className="font-bold text-xl text-yellow-500">{stats.collectiveStats.totalFiveStars.toLocaleString()}</p>
                                </div>
                                <div>
                                    <p className="text-muted-foreground text-sm">4★ Total</p>
                                    <p className="font-bold text-purple-500 text-xl">{stats.collectiveStats.totalFourStars.toLocaleString()}</p>
                                </div>
                                <div>
                                    <p className="text-muted-foreground text-sm">3★ Total</p>
                                    <p className="font-bold text-blue-500 text-xl">{stats.collectiveStats.totalThreeStars.toLocaleString()}</p>
                                </div>
                            </div>
                        </CardContent>
                    </Card>
                </InView>

                {/* Pull Timing Charts */}
                {stats.pullTiming && (
                    <div className="grid gap-4 md:grid-cols-2">
                        {/* Hourly Distribution */}
                        <InView
                            once
                            transition={{ duration: 0.5, ease: "easeOut", delay: 0.1 }}
                            variants={{
                                hidden: { opacity: 0, x: -20 },
                                visible: { opacity: 1, x: 0 },
                            }}
                        >
                            <Card>
                                <CardHeader>
                                    <CardTitle className="flex items-center gap-2">
                                        <Clock className="h-5 w-5" />
                                        Pulls by Hour
                                    </CardTitle>
                                    <CardDescription>When players are most active</CardDescription>
                                </CardHeader>
                                <CardContent>
                                    <ChartContainer
                                        className="h-64 w-full"
                                        config={{
                                            pulls: {
                                                label: "Pulls",
                                                color: "hsl(217, 91%, 60%)",
                                            },
                                        }}
                                    >
                                        <ResponsiveContainer height="100%" width="100%">
                                            <BarChart data={hourlyData}>
                                                <XAxis dataKey="hour" interval={2} stroke="hsl(var(--muted-foreground))" tick={{ fill: "hsl(var(--muted-foreground))", fontSize: 10 }} tickLine={false} />
                                                <YAxis stroke="hsl(var(--muted-foreground))" tick={{ fill: "hsl(var(--muted-foreground))", fontSize: 10 }} tickLine={false} width={40} />
                                                <ChartTooltip content={<ChartTooltipContent />} cursor={false} />
                                                <Bar dataKey="pulls" fill="#3b82f6" radius={[4, 4, 0, 0]} />
                                            </BarChart>
                                        </ResponsiveContainer>
                                    </ChartContainer>
                                </CardContent>
                            </Card>
                        </InView>

                        {/* Daily Distribution */}
                        <InView
                            once
                            transition={{ duration: 0.5, ease: "easeOut", delay: 0.2 }}
                            variants={{
                                hidden: { opacity: 0, x: 20 },
                                visible: { opacity: 1, x: 0 },
                            }}
                        >
                            <Card>
                                <CardHeader>
                                    <CardTitle className="flex items-center gap-2">
                                        <Calendar className="h-5 w-5" />
                                        Pulls by Day
                                    </CardTitle>
                                    <CardDescription>Weekly pull patterns</CardDescription>
                                </CardHeader>
                                <CardContent>
                                    <ChartContainer
                                        className="h-64 w-full"
                                        config={{
                                            pulls: {
                                                label: "Pulls",
                                                color: "hsl(142, 71%, 45%)",
                                            },
                                        }}
                                    >
                                        <ResponsiveContainer height="100%" width="100%">
                                            <BarChart data={dailyData}>
                                                <XAxis dataKey="day" stroke="hsl(var(--muted-foreground))" tick={{ fill: "hsl(var(--muted-foreground))", fontSize: 11 }} tickLine={false} />
                                                <YAxis stroke="hsl(var(--muted-foreground))" tick={{ fill: "hsl(var(--muted-foreground))", fontSize: 10 }} tickLine={false} width={40} />
                                                <ChartTooltip content={<ChartTooltipContent />} cursor={false} />
                                                <Bar dataKey="pulls" fill="#22c55e" radius={[4, 4, 0, 0]} />
                                            </BarChart>
                                        </ResponsiveContainer>
                                    </ChartContainer>
                                </CardContent>
                            </Card>
                        </InView>
                    </div>
                )}

                {/* Pull Activity Over Time (Line Chart) */}
                {dateData.length > 0 && (
                    <InView
                        once
                        transition={{ duration: 0.5, ease: "easeOut" }}
                        variants={{
                            hidden: { opacity: 0, y: 20 },
                            visible: { opacity: 1, y: 0 },
                        }}
                    >
                        <Card>
                            <CardHeader>
                                <CardTitle className="flex items-center gap-2">
                                    <TrendingUp className="h-5 w-5" />
                                    Pull Activity Over Time
                                </CardTitle>
                                <CardDescription>Historical daily pull volume from all contributing players</CardDescription>
                            </CardHeader>
                            <CardContent>
                                <ChartContainer
                                    className="h-72 w-full"
                                    config={{
                                        pulls: {
                                            label: "Pulls",
                                            color: "#8b5cf6",
                                        },
                                    }}
                                >
                                    <ResponsiveContainer height="100%" width="100%">
                                        <LineChart data={dateData}>
                                            <XAxis dataKey="date" interval="preserveStartEnd" stroke="hsl(var(--muted-foreground))" tick={{ fill: "hsl(var(--muted-foreground))", fontSize: 10 }} tickLine={false} />
                                            <YAxis stroke="hsl(var(--muted-foreground))" tick={{ fill: "hsl(var(--muted-foreground))", fontSize: 10 }} tickLine={false} width={50} />
                                            <ChartTooltip content={<ChartTooltipContent />} cursor={{ stroke: "hsl(var(--muted-foreground))", strokeDasharray: "5 5" }} />
                                            <Line activeDot={{ r: 5, fill: "#8b5cf6" }} dataKey="pulls" dot={{ fill: "#8b5cf6", strokeWidth: 0, r: 3 }} stroke="#8b5cf6" strokeWidth={2} type="monotone" />
                                        </LineChart>
                                    </ResponsiveContainer>
                                </ChartContainer>
                            </CardContent>
                        </Card>
                    </InView>
                )}

                {/* Most Common Operators */}
                <InView
                    once
                    transition={{ duration: 0.5, ease: "easeOut" }}
                    variants={{
                        hidden: { opacity: 0, y: 20 },
                        visible: { opacity: 1, y: 0 },
                    }}
                >
                    <Card>
                        <CardHeader>
                            <CardTitle className="flex items-center gap-2">
                                <Users className="h-5 w-5" />
                                Most Common Operators
                            </CardTitle>
                            <CardDescription>Top operators pulled by the community</CardDescription>
                        </CardHeader>
                        <CardContent>
                            <div className="grid gap-6 md:grid-cols-2 lg:grid-cols-4">
                                {([6, 5, 4, 3] as const).map((rarity) => {
                                    const operators = operatorsByRarity[rarity];
                                    const rarityColors = {
                                        6: { text: "text-orange-500", bg: "bg-orange-500/10", border: "border-orange-500/30" },
                                        5: { text: "text-yellow-500", bg: "bg-yellow-500/10", border: "border-yellow-500/30" },
                                        4: { text: "text-purple-500", bg: "bg-purple-500/10", border: "border-purple-500/30" },
                                        3: { text: "text-blue-500", bg: "bg-blue-500/10", border: "border-blue-500/30" },
                                    } as const;
                                    const colors = rarityColors[rarity];

                                    return (
                                        <div className="space-y-3" key={rarity}>
                                            <div className="flex items-center gap-2">
                                                <span className={`font-semibold text-sm ${colors.text}`}>{rarity}★ Operators</span>
                                            </div>
                                            {operators.length > 0 ? (
                                                <div className="space-y-2">
                                                    {operators.slice(0, 5).map((op, index) => (
                                                        <TooltipProvider key={op.charId}>
                                                            <Tooltip>
                                                                <TooltipTrigger asChild>
                                                                    <div className={`flex cursor-help items-center gap-2 rounded-lg border p-2 transition-colors hover:bg-accent/50 ${colors.bg} ${colors.border}`}>
                                                                        <span className="font-semibold text-muted-foreground text-xs">#{index + 1}</span>
                                                                        <div className="relative h-8 w-8 shrink-0 overflow-hidden rounded-md border border-border/50">
                                                                            <Image alt={op.charName} className="object-cover" fill sizes="32px" src={`/api/cdn/avatar/${encodeURIComponent(op.charId)}`} />
                                                                        </div>
                                                                        <div className="min-w-0 flex-1">
                                                                            <p className="truncate font-medium text-xs">{op.charName}</p>
                                                                        </div>
                                                                        <span className={`shrink-0 font-bold text-sm ${colors.text}`}>×{op.pullCount.toLocaleString()}</span>
                                                                    </div>
                                                                </TooltipTrigger>
                                                                <TooltipContent>
                                                                    <p className="font-semibold">{op.charName}</p>
                                                                    <p className="text-xs">
                                                                        {op.pullCount.toLocaleString()} pulls ({(op.percentage * 100).toFixed(2)}%)
                                                                    </p>
                                                                </TooltipContent>
                                                            </Tooltip>
                                                        </TooltipProvider>
                                                    ))}
                                                </div>
                                            ) : (
                                                <p className="py-4 text-center text-muted-foreground text-xs">No data</p>
                                            )}
                                        </div>
                                    );
                                })}
                            </div>
                        </CardContent>
                    </Card>
                </InView>

                {/* Data Source Notice */}
                <Card className="border-muted-foreground/20 bg-muted/20">
                    <CardContent className="pt-6">
                        <p className="text-center text-muted-foreground text-sm">Statistics are collected anonymously from users who opt-in to share their gacha data. Rates may vary from individual experiences. Data is updated regularly to reflect the latest community trends.</p>
                    </CardContent>
                </Card>
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
