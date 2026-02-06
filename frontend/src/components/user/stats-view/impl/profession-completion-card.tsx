"use client";

import { BarChart3, ChevronDown } from "lucide-react";
import { motion } from "motion/react";
import { useCallback, useMemo, useState } from "react";
import { ClassIcon } from "~/components/collection/operators/list/ui/impl/class-icon";
import { SubClassIcon } from "~/components/collection/operators/list/ui/impl/sub-class-icon";
import { AnimatedNumber } from "~/components/ui/motion-primitives/animated-number";
import { Card, CardContent, CardHeader, CardTitle } from "~/components/ui/shadcn/card";
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

// Local sort order for Class Breakdown: Medic above Supporter
const BREAKDOWN_SORT_ORDER: Record<string, number> = {
    PIONEER: 0,
    WARRIOR: 1,
    TANK: 2,
    SNIPER: 3,
    CASTER: 4,
    MEDIC: 5,
    SUPPORT: 6,
    SPECIAL: 7,
};

export function ProfessionCompletionCard({ professions }: ProfessionCompletionCardProps) {
    const sorted = useMemo(() => [...professions].sort((a, b) => (BREAKDOWN_SORT_ORDER[a.profession] ?? 99) - (BREAKDOWN_SORT_ORDER[b.profession] ?? 99)), [professions]);
    const [expanded, setExpanded] = useState<Set<string>>(new Set());

    const toggle = useCallback((profession: string) => {
        setExpanded((prev) => {
            const next = new Set(prev);
            if (next.has(profession)) {
                next.delete(profession);
            } else {
                next.add(profession);
            }
            return next;
        });
    }, []);

    return (
        <Card className="border-border/50 bg-linear-to-b from-card/60 to-card/40 backdrop-blur-sm">
            <CardHeader className="pb-4">
                <CardTitle className="flex items-center gap-2 font-medium text-muted-foreground text-sm">
                    <BarChart3 className="h-4 w-4" />
                    Class Breakdown
                </CardTitle>
            </CardHeader>
            <CardContent className="flex flex-col gap-1">
                {sorted.map((prof, index) => {
                    const colors = PROFESSION_COLORS[prof.profession] ?? {
                        color: "text-muted-foreground",
                        bgColor: "bg-muted/10",
                        progressColor: "bg-muted",
                    };
                    const isExpanded = expanded.has(prof.profession);
                    const hasSubProfessions = prof.subProfessions && prof.subProfessions.length > 0;

                    return (
                        <div key={prof.profession}>
                            <motion.button
                                animate={{ x: 0, opacity: 1 }}
                                className="group -mx-3 w-[calc(100%+1.5rem)] cursor-pointer rounded-lg px-3 py-2.5 text-left transition-colors hover:bg-muted/40"
                                initial={{ x: -20, opacity: 0 }}
                                onClick={() => toggle(prof.profession)}
                                transition={{ delay: index * 0.08, duration: 0.4, ease: "easeOut" }}
                                type="button"
                            >
                                <div className="flex items-center gap-2.5">
                                    <div className={`flex h-9 w-9 shrink-0 items-center justify-center rounded-md ${colors.bgColor}`}>
                                        <ClassIcon className="opacity-90" profession={prof.profession} size={24} />
                                    </div>
                                    <div className="flex w-full flex-col gap-1.5">
                                        <div className="flex items-center justify-between">
                                            <div className="flex items-center gap-1.5">
                                                <span className="font-medium text-sm">{prof.displayName}</span>
                                                {hasSubProfessions && (
                                                    <motion.div animate={{ rotate: isExpanded ? 180 : 0 }} transition={{ duration: 0.2 }}>
                                                        <ChevronDown className="h-3.5 w-3.5 text-muted-foreground/50" />
                                                    </motion.div>
                                                )}
                                            </div>
                                            <div className="flex items-center gap-1">
                                                <AnimatedNumber className={`font-semibold text-sm tabular-nums ${colors.color}`} springOptions={{ stiffness: 100, damping: 20 }} value={prof.owned} />
                                                <span className="text-muted-foreground/60 text-sm">/ {prof.total}</span>
                                            </div>
                                        </div>
                                        <div className="relative h-2 w-full overflow-hidden rounded-full bg-muted/50">
                                            <motion.div animate={{ width: `${prof.percentage}%` }} className={`h-full rounded-full ${colors.progressColor}`} initial={{ width: 0 }} transition={{ delay: index * 0.08 + 0.2, duration: 0.6, ease: "easeOut" }} />
                                        </div>
                                    </div>
                                </div>
                            </motion.button>

                            {hasSubProfessions && (
                                <div className="grid transition-[grid-template-rows] duration-300 ease-in-out" style={{ gridTemplateRows: isExpanded ? "1fr" : "0fr" }}>
                                    <div className="min-h-0 overflow-hidden">
                                        <div className="ml-[2.875rem] flex flex-col py-1">
                                            {prof.subProfessions.map((sub, subIndex) => {
                                                const isLast = subIndex === prof.subProfessions.length - 1;
                                                return (
                                                    <div className="relative flex" key={sub.subProfessionId}>
                                                        {/* Vertical line */}
                                                        <div className={`absolute left-0 w-px bg-muted-foreground/20 ${isLast ? "top-0 h-1/2" : "top-0 h-full"}`} />
                                                        {/* Horizontal branch */}
                                                        <div className="absolute top-1/2 left-0 h-px w-6 bg-muted-foreground/20" />
                                                        {/* Content */}
                                                        <div className="ml-6 flex flex-1 items-center justify-between gap-3 rounded-md px-2 py-1.5 transition-colors hover:bg-muted/20">
                                                            <div className="flex shrink-0 items-center gap-2">
                                                                <SubClassIcon className="shrink-0 opacity-70" size={16} subProfessionId={sub.subProfessionId} />
                                                                <span className="whitespace-nowrap text-foreground/70 text-xs">{sub.displayName.replace(/\s+\S+$/, "")}</span>
                                                            </div>
                                                            <div className="flex min-w-0 flex-1 items-center gap-2">
                                                                <div className="relative h-1.5 min-w-0 flex-1 overflow-hidden rounded-full bg-muted/40">
                                                                    <motion.div animate={{ width: `${sub.percentage}%` }} className={`h-full rounded-full ${colors.progressColor} opacity-70`} initial={{ width: 0 }} transition={{ duration: 0.4, ease: "easeOut" }} />
                                                                </div>
                                                                <span className={`w-10 shrink-0 text-right text-xs tabular-nums ${colors.color} opacity-80`}>
                                                                    {sub.owned}/{sub.total}
                                                                </span>
                                                            </div>
                                                        </div>
                                                    </div>
                                                );
                                            })}
                                        </div>
                                    </div>
                                </div>
                            )}
                        </div>
                    );
                })}
            </CardContent>
        </Card>
    );
}
