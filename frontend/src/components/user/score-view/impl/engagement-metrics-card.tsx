"use client";

import { Layers, Route, Sparkles, TrendingUp } from "lucide-react";
import { motion } from "motion/react";
import { AnimatedNumber } from "~/components/ui/motion-primitives/animated-number";
import { Card, CardContent, CardHeader, CardTitle } from "~/components/ui/shadcn/card";
import { Tooltip, TooltipContent, TooltipTrigger } from "~/components/ui/shadcn/tooltip";
import { cn } from "~/lib/utils";
import type { StoredUserGrade } from "~/types/api/impl/user";

interface EngagementMetricsCardProps {
    grade: StoredUserGrade;
    className?: string;
}

export function EngagementMetricsCard({ grade, className }: EngagementMetricsCardProps) {
    const { engagementMetrics } = grade;

    const metrics = [
        {
            key: "contentVariety",
            label: "Content Variety",
            value: engagementMetrics.contentVarietyScore,
            icon: Layers,
            color: "text-rose-400",
            bgColor: "bg-rose-500/10",
            barColor: "bg-rose-500",
            tooltipTitle: "Content Variety",
            tooltipContent: `Engaged with ${engagementMetrics.contentTypesEngaged} content types`,
            tooltipDetail: "Events, stories, side content & more",
        },
        {
            key: "roguelikeDepth",
            label: "Roguelike Depth",
            value: engagementMetrics.roguelikeDepthScore,
            icon: Route,
            color: "text-violet-400",
            bgColor: "bg-violet-500/10",
            barColor: "bg-violet-500",
            tooltipTitle: "Roguelike Depth",
            tooltipContent: "Integrated Strategies progress",
            tooltipDetail: "Relics, endings, and themes unlocked",
        },
        {
            key: "stageDiversity",
            label: "Stage Diversity",
            value: engagementMetrics.stageDiversityScore,
            icon: Sparkles,
            color: "text-amber-400",
            bgColor: "bg-amber-500/10",
            barColor: "bg-amber-500",
            tooltipTitle: "Stage Diversity",
            tooltipContent: "Variety of stages completed",
            tooltipDetail: "Main story, events, annihilation, etc.",
        },
        {
            key: "progressionDepth",
            label: "Progression",
            value: engagementMetrics.progressionDepthScore,
            icon: TrendingUp,
            color: "text-emerald-400",
            bgColor: "bg-emerald-500/10",
            barColor: "bg-emerald-500",
            tooltipTitle: "Progression Depth",
            tooltipContent: "Overall account progression",
            tooltipDetail: "Operator levels, masteries, modules",
        },
    ];

    return (
        <Card className={cn("flex h-full flex-col border-border/50 bg-gradient-to-tr from-card/70 to-card/50 backdrop-blur-sm", className)}>
            <CardHeader className="flex-none pb-3">
                <CardTitle className="flex items-center justify-between">
                    <span className="flex items-center gap-2 font-medium text-muted-foreground text-sm">
                        <Sparkles className="h-4 w-4" />
                        Engagement
                    </span>
                    <Tooltip>
                        <TooltipTrigger asChild>
                            <motion.span animate={{ opacity: 1 }} className="text-muted-foreground text-xs" initial={{ opacity: 0 }} transition={{ delay: 0.3 }}>
                                {engagementMetrics.contentTypesEngaged} types
                            </motion.span>
                        </TooltipTrigger>
                        <TooltipContent sideOffset={5} variant="dark">
                            <p className="font-medium">Content Types</p>
                            <p className="text-muted-foreground">You've engaged with {engagementMetrics.contentTypesEngaged} different content types</p>
                        </TooltipContent>
                    </Tooltip>
                </CardTitle>
            </CardHeader>
            <CardContent className="flex flex-1 flex-col justify-between">
                {/* Total engagement score */}
                <Tooltip>
                    <TooltipTrigger asChild>
                        <motion.div animate={{ y: 0, opacity: 1 }} className="mb-4 flex items-center justify-between rounded-lg bg-muted/30 px-3 py-2" initial={{ y: 10, opacity: 0 }} transition={{ delay: 0.1 }}>
                            <span className="text-muted-foreground text-sm">Total Engagement</span>
                            <AnimatedNumber className="font-semibold text-lg" springOptions={{ stiffness: 80, damping: 15 }} value={Math.round(engagementMetrics.totalEngagementScore)} />
                        </motion.div>
                    </TooltipTrigger>
                    <TooltipContent sideOffset={5} variant="dark">
                        <p className="font-medium">Total Engagement Score</p>
                        <p className="text-muted-foreground">Combined from all engagement metrics</p>
                    </TooltipContent>
                </Tooltip>

                {/* Individual metrics as compact bars */}
                <div className="flex flex-col gap-3">
                    {metrics.map((metric, index) => {
                        const Icon = metric.icon;
                        return (
                            <Tooltip key={metric.key}>
                                <TooltipTrigger asChild>
                                    <motion.div animate={{ x: 0, opacity: 1 }} className="-mx-2 flex items-center gap-2 rounded-md px-2 py-1.5 transition-colors hover:bg-muted/40" initial={{ x: -10, opacity: 0 }} transition={{ delay: 0.2 + index * 0.08 }}>
                                        <div className={cn("flex h-6 w-6 items-center justify-center rounded", metric.bgColor)}>
                                            <Icon className={cn("h-3.5 w-3.5", metric.color)} />
                                        </div>
                                        <div className="flex flex-1 flex-col gap-1">
                                            <div className="flex items-center justify-between">
                                                <span className="text-muted-foreground text-xs">{metric.label}</span>
                                                <span className={cn("font-medium text-xs tabular-nums", metric.color)}>{Math.round(metric.value)}</span>
                                            </div>
                                            <div className="relative h-1.5 w-full overflow-hidden rounded-full bg-muted/50">
                                                <motion.div animate={{ width: `${Math.min(metric.value, 100)}%` }} className={cn("h-full rounded-full", metric.barColor)} initial={{ width: 0 }} transition={{ delay: 0.3 + index * 0.1, duration: 0.5 }} />
                                            </div>
                                        </div>
                                    </motion.div>
                                </TooltipTrigger>
                                <TooltipContent className="max-w-55" sideOffset={5} variant="dark">
                                    <p className="font-medium">{metric.tooltipTitle}</p>
                                    <p className="text-muted-foreground">{metric.tooltipContent}</p>
                                    <p className="mt-1 text-2.5 text-muted-foreground/70">{metric.tooltipDetail}</p>
                                </TooltipContent>
                            </Tooltip>
                        );
                    })}
                </div>
            </CardContent>
        </Card>
    );
}
