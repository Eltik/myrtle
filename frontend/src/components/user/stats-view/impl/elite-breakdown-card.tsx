"use client";

import { ChevronUp } from "lucide-react";
import { motion } from "motion/react";
import { AnimatedNumber } from "~/components/ui/motion-primitives/animated-number";
import { Card, CardContent, CardHeader, CardTitle } from "~/components/ui/shadcn/card";
import { cn } from "~/lib/utils";

interface EliteBreakdownCardProps {
    e0: number;
    e1: number;
    e2: number;
    total: number;
    className?: string;
}

const ELITE_CONFIG = [
    { key: "e2", label: "Elite 2", color: "text-amber-400", bgColor: "bg-amber-500/15", progressColor: "bg-amber-500", dotColor: "bg-amber-400" },
    { key: "e1", label: "Elite 1", color: "text-sky-400", bgColor: "bg-sky-500/15", progressColor: "bg-sky-500", dotColor: "bg-sky-400" },
    { key: "e0", label: "Elite 0", color: "text-slate-400", bgColor: "bg-slate-500/15", progressColor: "bg-slate-500", dotColor: "bg-slate-400" },
] as const;

export function EliteBreakdownCard({ e0, e1, e2, total, className }: EliteBreakdownCardProps) {
    const counts: Record<string, number> = { e0, e1, e2 };

    return (
        <Card className={cn("flex h-full flex-col border-border/50 bg-linear-to-br from-card/80 to-card/40 backdrop-blur-sm", className)}>
            <CardHeader className="flex-none pb-2">
                <CardTitle className="flex items-center gap-2 font-medium text-muted-foreground text-sm">
                    <ChevronUp className="h-4 w-4" />
                    Elite Promotion
                </CardTitle>
            </CardHeader>
            <CardContent className="flex flex-1 flex-col justify-center gap-4">
                {ELITE_CONFIG.map((config, index) => {
                    const count = counts[config.key] ?? 0;
                    const percentage = total > 0 ? (count / total) * 100 : 0;

                    return (
                        <motion.div animate={{ x: 0, opacity: 1 }} className="space-y-1.5" initial={{ x: -15, opacity: 0 }} key={config.key} transition={{ delay: index * 0.1, duration: 0.4, ease: "easeOut" }}>
                            <div className="flex items-center justify-between">
                                <div className="flex items-center gap-2">
                                    <div className={cn("h-2.5 w-2.5 rounded-full", config.dotColor)} />
                                    <span className="font-medium text-sm">{config.label}</span>
                                </div>
                                <div className="flex items-center gap-1.5">
                                    <AnimatedNumber className={cn("font-semibold text-sm tabular-nums", config.color)} springOptions={{ stiffness: 100, damping: 20 }} value={count} />
                                    <span className="text-muted-foreground/60 text-xs">({percentage.toFixed(0)}%)</span>
                                </div>
                            </div>
                            <div className="relative h-1.5 w-full overflow-hidden rounded-full bg-muted/50">
                                <motion.div animate={{ width: `${percentage}%` }} className={cn("h-full rounded-full", config.progressColor)} initial={{ width: 0 }} transition={{ delay: 0.2 + index * 0.1, duration: 0.5, ease: "easeOut" }} />
                            </div>
                        </motion.div>
                    );
                })}
            </CardContent>
        </Card>
    );
}
