"use client";

import { BarChart3 } from "lucide-react";
import { motion } from "motion/react";
import { ClassIcon } from "~/components/collection/operators/list/ui/impl/class-icon";
import { AnimatedNumber } from "~/components/ui/motion-primitives/animated-number";
import { Card, CardContent, CardHeader, CardTitle } from "~/components/ui/shadcn/card";
import { Tooltip, TooltipContent, TooltipTrigger } from "~/components/ui/shadcn/tooltip";
import type { ProfessionStat } from "~/types/api/impl/stats";

interface ProfessionCompletionCardProps {
    professions: ProfessionStat[];
}

const PROFESSION_COLORS: Record<string, { color: string; bgColor: string; progressColor: string }> = {
    PIONEER: { color: "text-red-400", bgColor: "bg-red-500/10", progressColor: "bg-red-500" },
    WARRIOR: { color: "text-orange-400", bgColor: "bg-orange-500/10", progressColor: "bg-orange-500" },
    TANK: { color: "text-blue-400", bgColor: "bg-blue-500/10", progressColor: "bg-blue-500" },
    SNIPER: { color: "text-emerald-400", bgColor: "bg-emerald-500/10", progressColor: "bg-emerald-500" },
    CASTER: { color: "text-violet-400", bgColor: "bg-violet-500/10", progressColor: "bg-violet-500" },
    SUPPORT: { color: "text-cyan-400", bgColor: "bg-cyan-500/10", progressColor: "bg-cyan-500" },
    MEDIC: { color: "text-lime-400", bgColor: "bg-lime-500/10", progressColor: "bg-lime-500" },
    SPECIAL: { color: "text-amber-400", bgColor: "bg-amber-500/10", progressColor: "bg-amber-500" },
};

export function ProfessionCompletionCard({ professions }: ProfessionCompletionCardProps) {
    return (
        <Card className="border-border/50 bg-linear-to-b from-card/60 to-card/40 backdrop-blur-sm">
            <CardHeader className="pb-4">
                <CardTitle className="flex items-center gap-2 font-medium text-muted-foreground text-sm">
                    <BarChart3 className="h-4 w-4" />
                    Class Breakdown
                </CardTitle>
            </CardHeader>
            <CardContent className="flex flex-col gap-5">
                {professions.map((prof, index) => {
                    const colors = PROFESSION_COLORS[prof.profession] ?? {
                        color: "text-muted-foreground",
                        bgColor: "bg-muted/10",
                        progressColor: "bg-muted",
                    };

                    return (
                        <Tooltip key={prof.profession}>
                            <TooltipTrigger asChild>
                                <motion.div animate={{ x: 0, opacity: 1 }} className="group -mx-3 rounded-lg px-3 py-2 transition-colors hover:bg-muted/40" initial={{ x: -20, opacity: 0 }} transition={{ delay: index * 0.08, duration: 0.4, ease: "easeOut" }}>
                                    <div className="mb-2 flex items-center justify-between">
                                        <div className="flex items-center gap-2">
                                            <div className={`flex h-7 w-7 items-center justify-center rounded-md ${colors.bgColor}`}>
                                                <ClassIcon className="opacity-90" profession={prof.profession} size={18} />
                                            </div>
                                            <span className="font-medium text-sm">{prof.displayName}</span>
                                        </div>
                                        <div className="flex flex-col items-end gap-0.5">
                                            <div className="flex items-center gap-1">
                                                <AnimatedNumber className={`font-semibold text-sm tabular-nums ${colors.color}`} springOptions={{ stiffness: 100, damping: 20 }} value={prof.owned} />
                                                <span className="text-muted-foreground/60 text-sm">/ {prof.total}</span>
                                            </div>
                                        </div>
                                    </div>

                                    <div className="relative h-2 w-full overflow-hidden rounded-full bg-muted/50">
                                        <motion.div animate={{ width: `${prof.percentage}%` }} className={`h-full rounded-full ${colors.progressColor}`} initial={{ width: 0 }} transition={{ delay: index * 0.08 + 0.2, duration: 0.6, ease: "easeOut" }} />
                                    </div>
                                </motion.div>
                            </TooltipTrigger>
                            <TooltipContent className="max-w-55" sideOffset={5} variant="dark">
                                <p className="font-medium">{prof.displayName}</p>
                                <p className="text-muted-foreground">
                                    {prof.owned} of {prof.total} operators collected
                                </p>
                                <p className="mt-1 text-[0.625rem] text-muted-foreground/70">{prof.percentage.toFixed(1)}% completion</p>
                            </TooltipContent>
                        </Tooltip>
                    );
                })}
            </CardContent>
        </Card>
    );
}
