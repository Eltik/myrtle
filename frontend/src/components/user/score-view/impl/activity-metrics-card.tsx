"use client";

import { Activity, Calendar, Clock } from "lucide-react";
import { motion } from "motion/react";
import { AnimatedNumber } from "~/components/ui/motion-primitives/animated-number";
import { Card, CardContent, CardHeader, CardTitle } from "~/components/ui/shadcn/card";
import { Tooltip, TooltipContent, TooltipTrigger } from "~/components/ui/shadcn/tooltip";
import { cn } from "~/lib/utils";
import type { StoredUserGrade } from "~/types/api/impl/user";
import { CircularProgress } from "./circular-progress";

interface ActivityMetricsCardProps {
    grade: StoredUserGrade;
    className?: string;
}

export function ActivityMetricsCard({ grade, className }: ActivityMetricsCardProps) {
    const { activityMetrics, accountAgeDays } = grade;

    const metrics = [
        {
            key: "loginRecency",
            label: "Recency",
            value: activityMetrics.loginRecencyScore,
            icon: Clock,
            color: "stroke-sky-500",
            tooltipTitle: "Login Recency",
            tooltipContent: activityMetrics.daysSinceLogin === 0 ? "Logged in today" : activityMetrics.daysSinceLogin === 1 ? "Last login: Yesterday" : `Last login: ${activityMetrics.daysSinceLogin} days ago`,
            tooltipDetail: "Higher score = more recent activity",
        },
        {
            key: "loginFrequency",
            label: "Frequency",
            value: activityMetrics.loginFrequencyScore,
            icon: Activity,
            color: "stroke-emerald-500",
            tooltipTitle: "Login Frequency",
            tooltipContent: activityMetrics.checkInsThisCycle ? `${activityMetrics.checkInsThisCycle}/${activityMetrics.checkInCycleLength} check-ins this cycle` : "Based on login patterns",
            tooltipDetail: `${Math.round(activityMetrics.checkInCompletionRate ?? 0)}% check-in completion`,
        },
        {
            key: "consistency",
            label: "Consistency",
            value: activityMetrics.consistencyScore,
            icon: Calendar,
            color: "stroke-violet-500",
            tooltipTitle: "Mission Consistency",
            tooltipContent: "Based on daily/weekly mission completion",
            tooltipDetail: "Regularity of your play sessions",
        },
    ];

    return (
        <Card className={cn("flex h-full flex-col border-border/50 bg-linear-to-bl from-card/70 to-card/50 backdrop-blur-sm", className)}>
            <CardHeader className="flex-none pb-3">
                <CardTitle className="flex items-center justify-between">
                    <span className="flex items-center gap-2 font-medium text-muted-foreground text-sm">
                        <Activity className="h-4 w-4" />
                        Activity
                    </span>
                    <Tooltip>
                        <TooltipTrigger asChild>
                            <motion.span animate={{ opacity: 1 }} className="text-muted-foreground text-xs" initial={{ opacity: 0 }} transition={{ delay: 0.3 }}>
                                {accountAgeDays} days
                            </motion.span>
                        </TooltipTrigger>
                        <TooltipContent sideOffset={5} variant="dark">
                            <p className="font-medium">Account Age</p>
                            <p className="text-muted-foreground">Your account is {accountAgeDays} days old</p>
                        </TooltipContent>
                    </Tooltip>
                </CardTitle>
            </CardHeader>
            <CardContent className="flex flex-1 flex-col justify-between">
                {/* Total activity score */}
                <Tooltip>
                    <TooltipTrigger asChild>
                        <motion.div animate={{ y: 0, opacity: 1 }} className="mb-4 flex items-center justify-between rounded-lg bg-muted/30 px-3 py-2" initial={{ y: 10, opacity: 0 }} transition={{ delay: 0.1 }}>
                            <span className="text-muted-foreground text-sm">Total Activity</span>
                            <AnimatedNumber className="font-semibold text-lg" springOptions={{ stiffness: 80, damping: 15 }} value={Math.round(activityMetrics.totalActivityScore)} />
                        </motion.div>
                    </TooltipTrigger>
                    <TooltipContent sideOffset={5} variant="dark">
                        <p className="font-medium">Total Activity Score</p>
                        <p className="text-muted-foreground">Combined from recency, frequency & consistency</p>
                    </TooltipContent>
                </Tooltip>

                {/* Individual metrics */}
                <div className="flex items-center justify-around gap-2">
                    {metrics.map((metric, index) => (
                        <Tooltip key={metric.key}>
                            <TooltipTrigger asChild>
                                <motion.div animate={{ scale: 1, opacity: 1 }} className="flex flex-col items-center gap-1 rounded-lg px-3 py-2 transition-colors hover:bg-muted/40" initial={{ scale: 0.8, opacity: 0 }} transition={{ delay: 0.2 + index * 0.1 }}>
                                    <CircularProgress delay={0.3 + index * 0.15} max={100} progressClassName={cn(metric.color)} size={52} strokeWidth={4} value={metric.value} />
                                    <span className="text-muted-foreground text-xs">{metric.label}</span>
                                </motion.div>
                            </TooltipTrigger>
                            <TooltipContent className="max-w-50" sideOffset={5} variant="dark">
                                <p className="font-medium">{metric.tooltipTitle}</p>
                                <p className="text-muted-foreground">{metric.tooltipContent}</p>
                                <p className="mt-1 text-[0.625rem] text-muted-foreground/70">{metric.tooltipDetail}</p>
                            </TooltipContent>
                        </Tooltip>
                    ))}
                </div>
            </CardContent>
        </Card>
    );
}
