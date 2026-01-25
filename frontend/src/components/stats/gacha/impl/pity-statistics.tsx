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
                            <div className="flex items-center gap-3">
                                <div className="flex h-12 w-12 items-center justify-center rounded-lg bg-orange-500/10">
                                    <Dices className="h-6 w-6 text-orange-500" />
                                </div>
                                <div>
                                    <p className="font-semibold text-muted-foreground text-sm">6-Star Pity</p>
                                    <p className="font-bold text-2xl text-orange-500">
                                        <AnimatedNumber decimals={1} springOptions={{ bounce: 0, duration: 2000 }} value={averagePullsToSixStar} />
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
                                    <span className="font-semibold text-orange-500">{averagePullsToSixStar.toFixed(1)} pulls</span>
                                </div>
                                <Separator className="my-2" />
                                <p className="text-muted-foreground text-xs">{averagePullsToSixStar < 50 ? `Community pulls ${(50 - averagePullsToSixStar).toFixed(1)} pulls better than expected!` : `Community pulls ${(averagePullsToSixStar - 50).toFixed(1)} pulls more than expected.`}</p>
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
                                        <AnimatedNumber decimals={1} springOptions={{ bounce: 0, duration: 2000 }} value={averagePullsToFiveStar} />
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
                                    <span className="font-semibold text-yellow-500">{averagePullsToFiveStar.toFixed(1)} pulls</span>
                                </div>
                                <Separator className="my-2" />
                                <p className="text-muted-foreground text-xs">{averagePullsToFiveStar < 12.5 ? `Community pulls ${(12.5 - averagePullsToFiveStar).toFixed(1)} pulls better than expected!` : `Community pulls ${(averagePullsToFiveStar - 12.5).toFixed(1)} pulls more than expected.`}</p>
                            </div>
                        </div>
                    </div>
                </CardContent>
            </Card>
        </InView>
    );
}
