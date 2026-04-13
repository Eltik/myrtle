"use client";

import { Sparkles } from "lucide-react";
import { motion } from "motion/react";
import { AnimatedNumber } from "~/components/ui/motion-primitives/animated-number";
import { Card, CardContent, CardHeader, CardTitle } from "~/components/ui/shadcn/card";
import { Tooltip, TooltipContent, TooltipTrigger } from "~/components/ui/shadcn/tooltip";
import { cn } from "~/lib/utils";

interface MasteryStatsCardProps {
    m3Count: number;
    m6Count: number;
    m9Count: number;
    totalMasteryLevels: number;
    maxPossibleMasteryLevels: number;
    className?: string;
}

export function MasteryStatsCard({ m3Count, m6Count, m9Count, totalMasteryLevels, maxPossibleMasteryLevels, className }: MasteryStatsCardProps) {
    const masteryPercentage = maxPossibleMasteryLevels > 0 ? (totalMasteryLevels / maxPossibleMasteryLevels) * 100 : 0;

    return (
        <Card className={cn("flex h-full flex-col border-border/50 bg-linear-to-br from-card/80 to-card/40 backdrop-blur-sm", className)}>
            <CardHeader className="flex-none pb-2">
                <CardTitle className="flex items-center gap-2 font-medium text-muted-foreground text-sm">
                    <Sparkles className="h-4 w-4" />
                    Skill Mastery
                </CardTitle>
            </CardHeader>
            <CardContent className="flex flex-1 flex-col justify-center gap-4">
                {/* M3, M6, M9 counts */}
                <div className="grid grid-cols-3 gap-3">
                    <Tooltip>
                        <TooltipTrigger asChild>
                            <motion.div animate={{ y: 0, opacity: 1 }} className="flex flex-col items-center gap-1 rounded-lg bg-muted/30 p-3" initial={{ y: 10, opacity: 0 }} transition={{ delay: 0.1, duration: 0.4 }}>
                                <AnimatedNumber className="font-bold text-2xl text-rose-400 tabular-nums" springOptions={{ stiffness: 80, damping: 20 }} value={m3Count} />
                                <span className="text-muted-foreground text-xs">M3</span>
                            </motion.div>
                        </TooltipTrigger>
                        <TooltipContent sideOffset={5} variant="dark">
                            <p className="font-medium">M3 Operators</p>
                            <p className="text-muted-foreground">Operators with at least one skill at Mastery 3</p>
                        </TooltipContent>
                    </Tooltip>

                    <Tooltip>
                        <TooltipTrigger asChild>
                            <motion.div animate={{ y: 0, opacity: 1 }} className="flex flex-col items-center gap-1 rounded-lg bg-muted/30 p-3" initial={{ y: 10, opacity: 0 }} transition={{ delay: 0.15, duration: 0.4 }}>
                                <AnimatedNumber className="font-bold text-2xl text-amber-400 tabular-nums" springOptions={{ stiffness: 80, damping: 20 }} value={m6Count} />
                                <span className="text-muted-foreground text-xs">M6</span>
                            </motion.div>
                        </TooltipTrigger>
                        <TooltipContent sideOffset={5} variant="dark">
                            <p className="font-medium">M6 Operators</p>
                            <p className="text-muted-foreground">Operators with 2 skills at Mastery 3</p>
                        </TooltipContent>
                    </Tooltip>

                    <Tooltip>
                        <TooltipTrigger asChild>
                            <motion.div animate={{ y: 0, opacity: 1 }} className="flex flex-col items-center gap-1 rounded-lg bg-muted/30 p-3" initial={{ y: 10, opacity: 0 }} transition={{ delay: 0.2, duration: 0.4 }}>
                                <AnimatedNumber className="font-bold text-2xl text-violet-400 tabular-nums" springOptions={{ stiffness: 80, damping: 20 }} value={m9Count} />
                                <span className="text-muted-foreground text-xs">M9</span>
                            </motion.div>
                        </TooltipTrigger>
                        <TooltipContent sideOffset={5} variant="dark">
                            <p className="font-medium">M9 Operators</p>
                            <p className="text-muted-foreground">Operators with all 3 skills at Mastery 3</p>
                        </TooltipContent>
                    </Tooltip>
                </div>

                {/* Total mastery progress */}
                <motion.div animate={{ opacity: 1 }} className="space-y-2" initial={{ opacity: 0 }} transition={{ delay: 0.3 }}>
                    <div className="flex items-center justify-between text-xs">
                        <span className="text-muted-foreground">Total Mastery Levels</span>
                        <span className="font-medium tabular-nums">
                            <AnimatedNumber className="text-rose-400" springOptions={{ stiffness: 100, damping: 20 }} value={totalMasteryLevels} />
                            <span className="text-muted-foreground/60"> / {maxPossibleMasteryLevels}</span>
                        </span>
                    </div>
                    <div className="relative h-1.5 w-full overflow-hidden rounded-full bg-muted/50">
                        <motion.div animate={{ width: `${masteryPercentage}%` }} className="h-full rounded-full bg-rose-500" initial={{ width: 0 }} transition={{ delay: 0.4, duration: 0.6, ease: "easeOut" }} />
                    </div>
                    <p className="text-center text-[0.625rem] text-muted-foreground/70">{masteryPercentage.toFixed(1)}% of max possible (E2 operators only)</p>
                </motion.div>
            </CardContent>
        </Card>
    );
}
