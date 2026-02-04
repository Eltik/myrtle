"use client";

import { BarChart3, HelpCircle, Sparkles, Star, Target, TrendingUp, Users } from "lucide-react";
import Image from "next/image";
import { Card, CardContent, CardHeader, CardTitle } from "~/components/ui/shadcn/card";
import { Progress } from "~/components/ui/shadcn/progress";
import { Separator } from "~/components/ui/shadcn/separator";
import { Skeleton } from "~/components/ui/shadcn/skeleton";
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "~/components/ui/shadcn/tooltip";
import { calculateAvgPullsPerSixStar, calculatePity, calculateStats, countSixStarsInSoftPity, formatRate, getAllRecords, getMostCommonOperatorsByRarity, HARD_PITY, parseRarity, SOFT_PITY_START } from "~/lib/gacha-utils";
import type { GachaRecords } from "~/types/api";

interface StatsOverviewProps {
    records: GachaRecords | null;
    loading?: boolean;
}

export function StatsOverview({ records, loading }: StatsOverviewProps) {
    if (loading) {
        return (
            <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
                {Array.from({ length: 4 }).map((_, i) => (
                    // biome-ignore lint/suspicious/noArrayIndexKey: Static skeleton placeholders
                    <Skeleton className="h-32" key={i} />
                ))}
            </div>
        );
    }

    if (!records) {
        return null;
    }

    const allRecords = getAllRecords(records);
    const stats = calculateStats(allRecords);

    // Expected rates for comparison
    const expectedSixStarRate = 0.02; // 2%
    const expectedFiveStarRate = 0.08; // 8%

    // Calculate average pulls per 6-star
    const avgPullsPerSixStar = calculateAvgPullsPerSixStar(stats.totalPulls, stats.sixStarCount);

    // Count 6-stars obtained in soft pity
    const softPitySixStars = countSixStarsInSoftPity(allRecords);

    // Calculate per-banner pity
    const limitedPity = calculatePity(records.limited.records);
    const regularPity = calculatePity(records.regular.records);
    const specialPity = calculatePity(records.special.records);

    // Calculate per-banner 6-star counts
    const limitedSixStarCount = records.limited.records.filter((r) => parseRarity(r.star) === 6).length;
    const regularSixStarCount = records.regular.records.filter((r) => parseRarity(r.star) === 6).length;
    const specialSixStarCount = records.special.records.filter((r) => parseRarity(r.star) === 6).length;

    // Calculate per-banner averages
    const limitedAvg = limitedSixStarCount > 0 ? records.limited.total / limitedSixStarCount : 0;
    const regularAvg = regularSixStarCount > 0 ? records.regular.total / regularSixStarCount : 0;
    const specialAvg = specialSixStarCount > 0 ? records.special.total / specialSixStarCount : 0;

    // Get most common operators by rarity
    const mostCommonOperators = getMostCommonOperatorsByRarity(allRecords, 3);

    return (
        <div className="space-y-4">
            {/* Main Stats Grid */}
            <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
                {/* Total Pulls */}
                <Card>
                    <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                        <CardTitle className="font-medium text-sm">Total Pulls</CardTitle>
                        <BarChart3 className="h-4 w-4 text-muted-foreground" />
                    </CardHeader>
                    <CardContent>
                        <div className="font-bold text-2xl">{stats.totalPulls.toLocaleString()}</div>
                        <p className="text-muted-foreground text-xs">Across all banners</p>
                    </CardContent>
                </Card>

                {/* 6-Star Count */}
                <Card>
                    <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                        <CardTitle className="font-medium text-sm">6-Star Operators</CardTitle>
                        <Sparkles className="h-4 w-4 text-orange-500" />
                    </CardHeader>
                    <CardContent>
                        <div className="font-bold text-2xl text-orange-500">{stats.sixStarCount}</div>
                        <p className="text-muted-foreground text-xs">{formatRate(stats.sixStarRate)} rate</p>
                    </CardContent>
                </Card>

                {/* 5-Star Count */}
                <Card>
                    <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                        <CardTitle className="font-medium text-sm">5-Star Operators</CardTitle>
                        <Star className="h-4 w-4 text-yellow-500" />
                    </CardHeader>
                    <CardContent>
                        <div className="font-bold text-2xl text-yellow-500">{stats.fiveStarCount}</div>
                        <p className="text-muted-foreground text-xs">{formatRate(stats.fiveStarRate)} rate</p>
                    </CardContent>
                </Card>

                {/* Current Pity */}
                <Card>
                    <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                        <CardTitle className="font-medium text-sm">Current Pity</CardTitle>
                        <TrendingUp className="h-4 w-4 text-muted-foreground" />
                    </CardHeader>
                    <CardContent>
                        <div className="font-bold text-2xl">{stats.pityCount}</div>
                        <p className="text-muted-foreground text-xs">Pulls since last 6★</p>
                    </CardContent>
                </Card>
            </div>

            {/* Detailed Rate Comparison */}
            <Card>
                <CardHeader>
                    <CardTitle>Pull Rate Analysis</CardTitle>
                </CardHeader>
                <CardContent className="space-y-4">
                    {/* 6-Star Rate */}
                    <div className="space-y-2">
                        <div className="flex items-center justify-between text-sm">
                            <span className="font-medium">6-Star Rate</span>
                            <span className="text-muted-foreground">
                                {formatRate(stats.sixStarRate)} (Expected: {formatRate(expectedSixStarRate)})
                            </span>
                        </div>
                        <Progress className="h-2" value={(stats.sixStarRate / expectedSixStarRate) * 100} />
                        <p className="text-muted-foreground text-xs">
                            {stats.sixStarRate >= expectedSixStarRate ? "Above" : "Below"} expected rate by {Math.abs((stats.sixStarRate - expectedSixStarRate) * 100).toFixed(2)}%
                        </p>
                    </div>

                    <Separator />

                    {/* 5-Star Rate */}
                    <div className="space-y-2">
                        <div className="flex items-center justify-between text-sm">
                            <span className="font-medium">5-Star Rate</span>
                            <span className="text-muted-foreground">
                                {formatRate(stats.fiveStarRate)} (Expected: {formatRate(expectedFiveStarRate)})
                            </span>
                        </div>
                        <Progress className="h-2" value={(stats.fiveStarRate / expectedFiveStarRate) * 100} />
                        <p className="text-muted-foreground text-xs">
                            {stats.fiveStarRate >= expectedFiveStarRate ? "Above" : "Below"} expected rate by {Math.abs((stats.fiveStarRate - expectedFiveStarRate) * 100).toFixed(2)}%
                        </p>
                    </div>

                    <Separator />

                    {/* Rarity Breakdown */}
                    <div className="grid grid-cols-4 gap-4 text-center">
                        <div>
                            <p className="font-bold text-2xl text-orange-500">{stats.sixStarCount}</p>
                            <p className="text-muted-foreground text-xs">6★</p>
                        </div>
                        <div>
                            <p className="font-bold text-2xl text-yellow-500">{stats.fiveStarCount}</p>
                            <p className="text-muted-foreground text-xs">5★</p>
                        </div>
                        <div>
                            <p className="font-bold text-2xl text-purple-500">{stats.fourStarCount}</p>
                            <p className="text-muted-foreground text-xs">4★</p>
                        </div>
                        <div>
                            <p className="font-bold text-2xl text-blue-500">{stats.threeStarCount}</p>
                            <p className="text-muted-foreground text-xs">3★</p>
                        </div>
                    </div>
                </CardContent>
            </Card>

            {/* Pity System Analysis */}
            <div className="grid gap-4 md:grid-cols-2">
                {/* Average Pulls & Pity Info Card */}
                <Card>
                    <CardHeader>
                        <div className="flex items-center gap-2">
                            <CardTitle>Pity Analysis</CardTitle>
                            <TooltipProvider>
                                <Tooltip>
                                    <TooltipTrigger>
                                        <HelpCircle className="h-4 w-4 text-muted-foreground" />
                                    </TooltipTrigger>
                                    <TooltipContent className="max-w-xs">
                                        <p className="font-semibold">How Pity Works:</p>
                                        <ul className="mt-1 list-disc space-y-1 pl-4 text-xs">
                                            <li>
                                                <strong>Base rate:</strong> 2% for 6★
                                            </li>
                                            <li>
                                                <strong>Soft pity (pull 50+):</strong> Rate increases by ~2% per pull
                                            </li>
                                            <li>
                                                <strong>Hard pity (pull 99):</strong> Guaranteed 6★
                                            </li>
                                        </ul>
                                    </TooltipContent>
                                </Tooltip>
                            </TooltipProvider>
                        </div>
                    </CardHeader>
                    <CardContent className="space-y-4">
                        {/* Average Pulls per 6-Star */}
                        <div className="flex items-center justify-between">
                            <div className="flex items-center gap-2">
                                <Target className="h-4 w-4 text-orange-500" />
                                <span className="font-medium text-sm">Avg Pulls per 6★</span>
                            </div>
                            <span className="font-bold text-lg text-orange-500">{stats.sixStarCount > 0 ? avgPullsPerSixStar.toFixed(1) : "N/A"}</span>
                        </div>

                        <Separator />

                        {/* Overall Pity Progress */}
                        <div className="space-y-2">
                            <div className="flex items-center justify-between text-sm">
                                <span className="font-medium">Overall Pity Progress</span>
                                <span className="text-muted-foreground">
                                    {stats.pityCount} / {HARD_PITY}
                                </span>
                            </div>
                            <Progress className="h-2" value={(stats.pityCount / HARD_PITY) * 100} />
                            <p className="text-muted-foreground text-xs">{stats.pityCount >= SOFT_PITY_START ? "In soft pity range! Increased 6★ chance." : `${SOFT_PITY_START - stats.pityCount} pulls until soft pity`}</p>
                        </div>

                        <Separator />

                        {/* Soft Pity Statistics */}
                        <div className="grid grid-cols-2 gap-4 text-center">
                            <div>
                                <p className="font-bold text-lg">{softPitySixStars}</p>
                                <p className="text-muted-foreground text-xs">6★ in soft pity</p>
                            </div>
                            <div>
                                <p className="font-bold text-lg">{stats.sixStarCount > 0 ? `${((softPitySixStars / stats.sixStarCount) * 100).toFixed(0)}%` : "N/A"}</p>
                                <p className="text-muted-foreground text-xs">of 6★ from soft pity</p>
                            </div>
                        </div>
                    </CardContent>
                </Card>

                {/* Per-Banner Pity Card */}
                <Card>
                    <CardHeader>
                        <CardTitle>Banner Pity Status</CardTitle>
                    </CardHeader>
                    <CardContent className="space-y-4">
                        {/* Limited Banner */}
                        <div className="space-y-2">
                            <div className="flex items-center justify-between text-sm">
                                <span className="font-medium text-orange-500">Limited Banner</span>
                                <span className="text-muted-foreground">
                                    {limitedPity} / {HARD_PITY}
                                </span>
                            </div>
                            <Progress className="h-2" value={(limitedPity / HARD_PITY) * 100} />
                        </div>

                        {/* Regular Banner */}
                        <div className="space-y-2">
                            <div className="flex items-center justify-between text-sm">
                                <span className="font-medium text-blue-500">Regular Banner</span>
                                <span className="text-muted-foreground">
                                    {regularPity} / {HARD_PITY}
                                </span>
                            </div>
                            <Progress className="h-2" value={(regularPity / HARD_PITY) * 100} />
                        </div>

                        {/* Special Banner */}
                        <div className="space-y-2">
                            <div className="flex items-center justify-between text-sm">
                                <span className="font-medium text-purple-500">Special Banner</span>
                                <span className="text-muted-foreground">
                                    {specialPity} / {HARD_PITY}
                                </span>
                            </div>
                            <Progress className="h-2" value={(specialPity / HARD_PITY) * 100} />
                        </div>

                        <Separator />

                        {/* Per-Banner Averages */}
                        <div className="grid grid-cols-3 gap-4 text-center text-xs">
                            <div>
                                <p className="font-semibold text-orange-500">{limitedAvg > 0 ? limitedAvg.toFixed(1) : "N/A"}</p>
                                <p className="text-muted-foreground">Limited avg</p>
                            </div>
                            <div>
                                <p className="font-semibold text-blue-500">{regularAvg > 0 ? regularAvg.toFixed(1) : "N/A"}</p>
                                <p className="text-muted-foreground">Regular avg</p>
                            </div>
                            <div>
                                <p className="font-semibold text-purple-500">{specialAvg > 0 ? specialAvg.toFixed(1) : "N/A"}</p>
                                <p className="text-muted-foreground">Special avg</p>
                            </div>
                        </div>
                    </CardContent>
                </Card>
            </div>

            {/* Most Common Operators */}
            <Card>
                <CardHeader>
                    <div className="flex items-center gap-2">
                        <Users className="h-5 w-5 text-muted-foreground" />
                        <CardTitle>Most Common Operators</CardTitle>
                    </div>
                </CardHeader>
                <CardContent>
                    <div className="grid gap-6 md:grid-cols-2 lg:grid-cols-4">
                        {([6, 5, 4, 3] as const).map((rarity) => {
                            const operators = mostCommonOperators[rarity] ?? [];
                            const rarityColors = {
                                6: { text: "text-orange-500", bg: "bg-orange-500/10" },
                                5: { text: "text-yellow-500", bg: "bg-yellow-500/10" },
                                4: { text: "text-purple-500", bg: "bg-purple-500/10" },
                                3: { text: "text-blue-500", bg: "bg-blue-500/10" },
                            } as const;
                            const colors = rarityColors[rarity];

                            return (
                                <div className="space-y-3" key={rarity}>
                                    <div className="flex items-center gap-2">
                                        <span className={`font-semibold text-sm ${colors.text}`}>{rarity}★ Operators</span>
                                    </div>
                                    {operators.length > 0 ? (
                                        <div className="space-y-2">
                                            {operators.map((op, index) => (
                                                <div className={`flex items-center gap-2 rounded-md p-2 ${colors.bg}`} key={op.charId}>
                                                    <span className="font-semibold text-muted-foreground text-xs">#{index + 1}</span>
                                                    <div className="relative h-8 w-8 shrink-0 overflow-hidden rounded-md border border-border/50">
                                                        <Image alt={op.charName} className="object-cover" fill sizes="32px" src={`/api/cdn/avatar/${encodeURIComponent(op.charId)}`} />
                                                    </div>
                                                    <div className="min-w-0 flex-1">
                                                        <p className="truncate font-medium text-xs">{op.charName}</p>
                                                    </div>
                                                    <span className={`shrink-0 font-bold text-sm ${colors.text}`}>×{op.count}</span>
                                                </div>
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
        </div>
    );
}
