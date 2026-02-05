import { Dices, Sparkles, Target } from "lucide-react";
import { AnimatedNumber } from "~/components/ui/motion-primitives/animated-number";
import { InView } from "~/components/ui/motion-primitives/in-view";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "~/components/ui/shadcn/card";
import { Separator } from "~/components/ui/shadcn/separator";

interface PityStatisticsProps {
    averagePullsToSixStar: number;
    averagePullsToFiveStar: number;
}

export function PityStatistics({ averagePullsToSixStar, averagePullsToFiveStar }: PityStatisticsProps) {
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
                        <Target className="h-5 w-5" />
                        Average Pity Statistics
                    </CardTitle>
                    <CardDescription>Community average pulls required to obtain rare operators</CardDescription>
                </CardHeader>
                <CardContent>
                    <div className="grid gap-6 md:grid-cols-2">
                        {/* 6-Star Pity */}
                        <div className="space-y-4">
                            <div className="flex items-center gap-2 sm:gap-3">
                                <div className="flex h-10 w-10 shrink-0 items-center justify-center rounded-lg bg-orange-500/10 sm:h-12 sm:w-12">
                                    <Dices className="h-5 w-5 text-orange-500 sm:h-6 sm:w-6" />
                                </div>
                                <div>
                                    <p className="font-medium text-[0.625rem] text-muted-foreground uppercase tracking-wider sm:text-xs">6-Star Pity</p>
                                    <p className="font-bold text-orange-500 text-xl tracking-tight sm:text-2xl">
                                        <AnimatedNumber decimals={1} springOptions={{ bounce: 0, duration: 2000 }} value={averagePullsToSixStar} />
                                        <span className="ml-1 font-normal text-muted-foreground text-sm sm:text-base">pulls</span>
                                    </p>
                                </div>
                            </div>
                            <div className="space-y-2 rounded-lg bg-orange-500/5 p-3 sm:p-4">
                                <div className="flex flex-wrap justify-between gap-x-2 text-xs sm:text-sm">
                                    <span className="text-muted-foreground">Expected pity:</span>
                                    <span>
                                        <span className="font-semibold tabular-nums">50</span>
                                        <span className="font-medium"> pulls </span>
                                        <span className="text-muted-foreground">(guaranteed at 99)</span>
                                    </span>
                                </div>
                                <div className="flex flex-wrap justify-between gap-x-2 text-xs sm:text-sm">
                                    <span className="text-muted-foreground">Community average:</span>
                                    <span className="font-semibold text-orange-500 tabular-nums">
                                        {averagePullsToSixStar.toFixed(1)} <span className="font-medium">pulls</span>
                                    </span>
                                </div>
                                <Separator className="my-2" />
                                <p className="text-muted-foreground text-xs">
                                    {averagePullsToSixStar < 50 ? (
                                        <>
                                            Community averages <span className="font-semibold text-orange-500 tabular-nums">{(50 - averagePullsToSixStar).toFixed(1)}</span> pulls better than expected!
                                        </>
                                    ) : (
                                        <>
                                            Community averages <span className="font-semibold text-orange-500 tabular-nums">{(averagePullsToSixStar - 50).toFixed(1)}</span> pulls more than expected.
                                        </>
                                    )}
                                </p>
                            </div>
                        </div>

                        {/* 5-Star Pity */}
                        <div className="space-y-4">
                            <div className="flex items-center gap-2 sm:gap-3">
                                <div className="flex h-10 w-10 shrink-0 items-center justify-center rounded-lg bg-yellow-500/10 sm:h-12 sm:w-12">
                                    <Sparkles className="h-5 w-5 text-yellow-500 sm:h-6 sm:w-6" />
                                </div>
                                <div>
                                    <p className="font-medium text-[0.625rem] text-muted-foreground uppercase tracking-wider sm:text-xs">5-Star Pity</p>
                                    <p className="font-bold text-xl text-yellow-500 tracking-tight sm:text-2xl">
                                        <AnimatedNumber decimals={1} springOptions={{ bounce: 0, duration: 2000 }} value={averagePullsToFiveStar} />
                                        <span className="ml-1 font-normal text-muted-foreground text-sm sm:text-base">pulls</span>
                                    </p>
                                </div>
                            </div>
                            <div className="space-y-2 rounded-lg bg-yellow-500/5 p-3 sm:p-4">
                                <div className="flex flex-wrap justify-between gap-x-2 text-xs sm:text-sm">
                                    <span className="text-muted-foreground">Expected rate:</span>
                                    <span>
                                        <span className="font-semibold tabular-nums">1</span>
                                        <span className="font-medium"> per </span>
                                        <span className="font-semibold tabular-nums">12.5</span>
                                        <span className="font-medium"> pulls </span>
                                        <span className="text-muted-foreground">(8%)</span>
                                    </span>
                                </div>
                                <div className="flex flex-wrap justify-between gap-x-2 text-xs sm:text-sm">
                                    <span className="text-muted-foreground">Community average:</span>
                                    <span className="font-semibold text-yellow-500 tabular-nums">
                                        {averagePullsToFiveStar.toFixed(1)} <span className="font-medium">pulls</span>
                                    </span>
                                </div>
                                <Separator className="my-2" />
                                <p className="text-muted-foreground text-xs">
                                    {averagePullsToFiveStar < 12.5 ? (
                                        <>
                                            Community averages <span className="font-semibold text-yellow-500 tabular-nums">{(12.5 - averagePullsToFiveStar).toFixed(1)}</span> pulls better than expected!
                                        </>
                                    ) : (
                                        <>
                                            Community averages <span className="font-semibold text-yellow-500 tabular-nums">{(averagePullsToFiveStar - 12.5).toFixed(1)}</span> pulls more than expected.
                                        </>
                                    )}
                                </p>
                            </div>
                        </div>
                    </div>
                </CardContent>
            </Card>
        </InView>
    );
}
