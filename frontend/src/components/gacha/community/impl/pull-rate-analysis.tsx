import { TrendingUp } from "lucide-react";
import { AnimatedNumber } from "~/components/ui/motion-primitives/animated-number";
import { InView } from "~/components/ui/motion-primitives/in-view";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "~/components/ui/shadcn/card";
import { Separator } from "~/components/ui/shadcn/separator";
import { formatRate } from "~/lib/gacha-utils";
import type { PullRates } from "~/types/api";
import { EXPECTED_RATES } from "./constants";

interface PullRateAnalysisProps {
    pullRates: PullRates;
    averagePullsToSixStar: number;
    averagePullsToFiveStar: number;
}

export function PullRateAnalysis({ pullRates, averagePullsToSixStar, averagePullsToFiveStar }: PullRateAnalysisProps) {
    return (
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
                                <span className={`hidden font-bold sm:block ${pullRates.sixStarRate >= EXPECTED_RATES[6] ? "text-green-500" : "text-yellow-500"}`}>{formatRate(pullRates.sixStarRate)}</span>
                                <span className="text-muted-foreground text-sm">(Expected: {formatRate(EXPECTED_RATES[6])})</span>
                            </div>
                        </div>
                        <div className="relative h-4 overflow-hidden rounded-full bg-secondary">
                            <div
                                className="h-full rounded-full bg-linear-to-r from-orange-500 to-orange-400 transition-all duration-1000"
                                style={{
                                    width: `${Math.min((pullRates.sixStarRate / EXPECTED_RATES[6]) * 100, 100)}%`,
                                }}
                            />
                        </div>
                        <p className="text-muted-foreground text-xs">
                            Community rate is {Math.abs((pullRates.sixStarRate - EXPECTED_RATES[6]) * 100).toFixed(2)}% {pullRates.sixStarRate >= EXPECTED_RATES[6] ? "above" : "below"} expected
                        </p>
                    </div>

                    <Separator />

                    {/* 5-Star Rate Comparison */}
                    <div className="space-y-2">
                        <div className="flex items-center justify-between">
                            <span className="font-semibold">5-Star Rate</span>
                            <div className="flex items-center gap-2">
                                <span className={`hidden font-bold sm:block ${pullRates.fiveStarRate >= EXPECTED_RATES[5] ? "text-green-500" : "text-yellow-500"}`}>{formatRate(pullRates.fiveStarRate)}</span>
                                <span className="text-muted-foreground text-sm">(Expected: {formatRate(EXPECTED_RATES[5])})</span>
                            </div>
                        </div>
                        <div className="relative h-4 overflow-hidden rounded-full bg-secondary">
                            <div
                                className="h-full rounded-full bg-linear-to-r from-yellow-500 to-yellow-400 transition-all duration-1000"
                                style={{
                                    width: `${Math.min((pullRates.fiveStarRate / EXPECTED_RATES[5]) * 100, 100)}%`,
                                }}
                            />
                        </div>
                        <p className="text-muted-foreground text-xs">
                            Community rate is {Math.abs((pullRates.fiveStarRate - EXPECTED_RATES[5]) * 100).toFixed(2)}% {pullRates.fiveStarRate >= EXPECTED_RATES[5] ? "above" : "below"} expected
                        </p>
                    </div>

                    <Separator />

                    {/* Average Pulls */}
                    <div className="grid gap-4 sm:grid-cols-2">
                        <div className="space-y-1 text-center">
                            <p className="text-muted-foreground text-sm">Avg Pulls per 6★</p>
                            <p className="font-bold text-3xl text-orange-500">
                                <AnimatedNumber decimals={1} springOptions={{ bounce: 0, duration: 2000 }} value={averagePullsToSixStar} />
                            </p>
                        </div>
                        <div className="space-y-1 text-center">
                            <p className="text-muted-foreground text-sm">Avg Pulls per 5★</p>
                            <p className="font-bold text-3xl text-yellow-500">
                                <AnimatedNumber decimals={1} springOptions={{ bounce: 0, duration: 2000 }} value={averagePullsToFiveStar} />
                            </p>
                        </div>
                    </div>
                </CardContent>
            </Card>
        </InView>
    );
}
