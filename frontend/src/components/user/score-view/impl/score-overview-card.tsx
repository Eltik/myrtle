"use client";

import { Info, TrendingUp } from "lucide-react";
import { motion } from "motion/react";
import { AnimatedNumber } from "~/components/ui/motion-primitives/animated-number";
import { Card, CardContent, CardHeader, CardTitle } from "~/components/ui/shadcn/card";
import { Tooltip, TooltipContent, TooltipTrigger } from "~/components/ui/shadcn/tooltip";
import { cn } from "~/lib/utils";
import type { StoredUserScore } from "~/types/api/impl/user";
import { SCORE_CATEGORY_CONFIG } from "./constants";

interface ScoreOverviewCardProps {
    scoreData: StoredUserScore;
    className?: string;
}

// Get the top 3 contributing categories
function getTopCategories(scoreData: StoredUserScore) {
    const categories = [
        { key: "operatorScore", value: scoreData.operatorScore },
        { key: "stageScore", value: scoreData.stageScore },
        { key: "roguelikeScore", value: scoreData.roguelikeScore },
        { key: "sandboxScore", value: scoreData.sandboxScore },
        { key: "medalScore", value: scoreData.medalScore },
        { key: "baseScore", value: scoreData.baseScore },
    ];

    return categories
        .sort((a, b) => b.value - a.value)
        .slice(0, 3)
        .map((cat) => ({
            ...cat,
            config: SCORE_CATEGORY_CONFIG[cat.key],
            percentage: scoreData.totalScore > 0 ? (cat.value / scoreData.totalScore) * 100 : 0,
        }));
}

export function ScoreOverviewCard({ scoreData, className }: ScoreOverviewCardProps) {
    const topCategories = getTopCategories(scoreData);

    return (
        <Card className={cn("flex h-full flex-col border-border/50 bg-gradient-to-tr from-card/70 to-card/50 backdrop-blur-sm", className)}>
            <CardHeader className="flex-none pb-2">
                <CardTitle className="flex items-center gap-2 font-medium text-muted-foreground text-sm">
                    <TrendingUp className="h-4 w-4" />
                    Total Score
                </CardTitle>
            </CardHeader>
            <CardContent className="flex flex-1 flex-col justify-between gap-4">
                {/* Main score display */}
                <motion.div animate={{ scale: 1, opacity: 1 }} className="flex flex-col items-center gap-1" initial={{ scale: 0.9, opacity: 0 }} transition={{ type: "spring", stiffness: 150, damping: 20 }}>
                    <AnimatedNumber className="font-bold text-4xl tabular-nums tracking-tight" springOptions={{ stiffness: 60, damping: 20 }} value={scoreData.totalScore} />
                    <p className="text-center text-muted-foreground text-xs">Combined across all categories</p>
                </motion.div>

                {/* Top contributors breakdown */}
                <motion.div animate={{ y: 0, opacity: 1 }} className="space-y-2" initial={{ y: 10, opacity: 0 }} transition={{ delay: 0.2 }}>
                    <div className="flex items-center justify-center gap-1">
                        <p className="text-muted-foreground/70 text-xs">Top Contributors</p>
                        <Tooltip>
                            <TooltipTrigger asChild>
                                <Info className="h-3 w-3 cursor-help text-muted-foreground/50" />
                            </TooltipTrigger>
                            <TooltipContent className="max-w-[220px]" sideOffset={5} variant="dark">
                                <p className="font-medium">Contribution to Total</p>
                                <p className="text-muted-foreground">Bars show each category's share of your total score, not progress toward max.</p>
                            </TooltipContent>
                        </Tooltip>
                    </div>
                    <div className="space-y-1">
                        {topCategories.map((cat, index) => {
                            const Icon = cat.config?.icon;
                            return (
                                <Tooltip key={cat.key}>
                                    <TooltipTrigger asChild>
                                        <motion.div animate={{ x: 0, opacity: 1 }} className="-mx-2 flex cursor-help items-center gap-2 rounded-md px-2 py-1.5 transition-colors hover:bg-muted/40" initial={{ x: -10, opacity: 0 }} transition={{ delay: 0.3 + index * 0.08 }}>
                                            {Icon && (
                                                <div className={cn("flex h-5 w-5 items-center justify-center rounded", cat.config?.bgColor)}>
                                                    <Icon className={cn("h-3 w-3", cat.config?.color)} />
                                                </div>
                                            )}
                                            <div className="min-w-0 flex-1">
                                                <div className="flex items-center justify-between text-xs">
                                                    <span className="truncate text-muted-foreground">{cat.config?.label}</span>
                                                    <span className={cn("font-medium tabular-nums", cat.config?.color)}>{cat.value.toLocaleString()}</span>
                                                </div>
                                                <div className="relative mt-0.5 h-1 w-full overflow-hidden rounded-full bg-muted/30">
                                                    <motion.div animate={{ width: `${Math.min(cat.percentage, 100)}%` }} className={cn("h-full rounded-full", cat.config?.progressColor)} initial={{ width: 0 }} transition={{ delay: 0.4 + index * 0.1, duration: 0.5 }} />
                                                </div>
                                            </div>
                                        </motion.div>
                                    </TooltipTrigger>
                                    <TooltipContent className="max-w-[200px]" sideOffset={5} variant="dark">
                                        <p className="font-medium">{cat.config?.label}</p>
                                        <p className="text-muted-foreground">{cat.percentage.toFixed(1)}% of total score</p>
                                        <p className="mt-1 text-[10px] text-muted-foreground/70">See Score Breakdown for full details</p>
                                    </TooltipContent>
                                </Tooltip>
                            );
                        })}
                    </div>
                </motion.div>
            </CardContent>
        </Card>
    );
}
