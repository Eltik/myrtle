import { useQuery } from "@tanstack/react-query";
import { BarChart3, ChevronDown, ChevronUp, Layers, Palette, Sparkles, Users } from "lucide-react";
import { useCallback, useMemo, useState } from "react";
import { ClassIcon, SubProfessionIcon } from "#/components/operators/list/impl/components/Icons";
import { Card, CardContent, CardHeader, CardTitle } from "#/components/ui/card";
import { Tooltip, TooltipContent, TooltipTrigger } from "#/components/ui/tooltip";
import { skinsQueryOptions } from "#/lib/api/skins";
import type { IRosterEntry } from "#/lib/api/user";
import { cn } from "#/lib/utils";
import type { IOperatorListItem } from "#/types/operators";
import { computeUserStats } from "./helpers";

interface IStatsViewProps {
    roster: IRosterEntry[];
    operatorsStatic: IOperatorListItem[];
}

const ELITE_CONFIG = [
    { key: "e2", label: "Elite 2", color: "text-amber-400", bgColor: "bg-amber-500/15", progressColor: "bg-amber-500", dotColor: "bg-amber-400" },
    { key: "e1", label: "Elite 1", color: "text-sky-400", bgColor: "bg-sky-500/15", progressColor: "bg-sky-500", dotColor: "bg-sky-400" },
    { key: "e0", label: "Elite 0", color: "text-slate-400", bgColor: "bg-slate-500/15", progressColor: "bg-slate-500", dotColor: "bg-slate-400" },
] as const;

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

export function StatsTab({ roster, operatorsStatic }: IStatsViewProps) {
    const { data: skinData } = useQuery(skinsQueryOptions());

    const stats = useMemo(() => computeUserStats(roster, operatorsStatic, skinData?.charSkins), [roster, operatorsStatic, skinData]);

    const maxSubNameLen = useMemo(() => {
        let max = 0;
        for (const prof of stats.professions) {
            for (const sub of prof.subProfessions) {
                const name = sub.displayName.replace(/\s+\S+$/, "");
                if (name.length > max) max = name.length;
            }
        }
        return max;
    }, [stats.professions]);

    const [expanded, setExpanded] = useState<Set<string>>(new Set());

    const toggle = useCallback((profession: string) => {
        setExpanded((prev) => {
            const next = new Set(prev);
            if (next.has(profession)) next.delete(profession);
            else next.add(profession);
            return next;
        });
    }, []);

    if (!skinData?.charSkins) {
        // skeleton
        return <></>;
    }

    const counts: Record<string, number> = { e0: stats.eliteBreakdown.e0, e1: stats.eliteBreakdown.e1, e2: stats.eliteBreakdown.e2 };

    return (
        <div className="grid gap-4 pb-8 sm:grid-cols-2">
            <div>
                <Card className="flex h-full flex-col border-border/50 bg-linear-to-br from-card/80 to-card/40 backdrop-blur-sm">
                    <CardHeader className="flex-none">
                        <CardTitle className="flex items-center gap-2 font-medium text-muted-foreground text-sm">
                            <Users className="h-4 w-4" />
                            Operator Collection
                        </CardTitle>
                    </CardHeader>
                    <CardContent className="flex flex-1 flex-col justify-between gap-4">
                        <div className="flex flex-col items-center gap-1">
                            <div className="flex items-baseline gap-1">
                                <span className="font-bold text-4xl tracking-tight">{stats.totalOwned}</span>
                                <span className="text-lg text-muted-foreground">/ {stats.totalAvailable}</span>
                            </div>
                            <p className="text-center text-muted-foreground text-xs">operators collected</p>
                        </div>

                        <div className="space-y-2">
                            <div className="flex items-center justify-between text-xs">
                                <span className="text-muted-foreground">Completion</span>
                                <span className="text-emerald-400">
                                    <span className="font-medium text-emerald-400 tabular-nums">{Math.round((stats.collectionPercentage + Number.EPSILON) * 100) / 100}</span>%
                                </span>
                            </div>
                            <div className="relative h-2.5 w-full overflow-hidden rounded-full bg-muted/50">
                                <div className="h-full rounded-full bg-emerald-500" style={{ width: `${stats.collectionPercentage}%` }} />
                            </div>
                        </div>
                    </CardContent>
                </Card>
            </div>
            <div>
                <Card className="flex h-full flex-col border-border/50 bg-linear-to-br from-card/80 to-card/40 backdrop-blur-sm">
                    <CardHeader className="flex-none pb-2">
                        <CardTitle className="flex items-center gap-2 font-medium text-muted-foreground text-sm">
                            <ChevronUp className="h-4 w-4" />
                            Elite Promotion
                        </CardTitle>
                    </CardHeader>
                    <CardContent className="flex flex-1 flex-col justify-center gap-4">
                        {ELITE_CONFIG.map((config) => {
                            const count = counts[config.key] ?? 0;
                            const percentage = stats.eliteBreakdown.total > 0 ? (count / stats.eliteBreakdown.total) * 100 : 0;

                            return (
                                <div className="space-y-1.5" key={config.key}>
                                    <div className="flex items-center justify-between">
                                        <div className="flex items-center gap-2">
                                            <div className={cn("h-2.5 w-2.5 rounded-full", config.dotColor)} />
                                            <span className="font-medium text-sm">{config.label}</span>
                                        </div>
                                        <div className="flex items-center gap-1.5">
                                            <span className={cn("font-semibold text-sm tabular-nums", config.color)}>{count}</span>
                                            <span className="text-muted-foreground/60 text-xs">({percentage.toFixed(0)}%)</span>
                                        </div>
                                    </div>
                                    <div className="relative h-1.5 w-full overflow-hidden rounded-full bg-muted/50">
                                        <div className={cn("h-full rounded-full", config.progressColor)} style={{ width: `${percentage}%` }} />
                                    </div>
                                </div>
                            );
                        })}
                    </CardContent>
                </Card>
            </div>
            <div className="sm:col-span-2">
                <div>
                    <Card className="border-border/50 bg-linear-to-b from-card/60 to-card/40 backdrop-blur-sm">
                        <CardHeader className="pb-4">
                            <CardTitle className="flex items-center gap-2 font-medium text-muted-foreground text-sm">
                                <BarChart3 className="h-4 w-4" />
                                Class Breakdown
                            </CardTitle>
                        </CardHeader>
                        <CardContent className="flex flex-col gap-1">
                            {stats.professions.map((prof) => {
                                const colors = PROFESSION_COLORS[prof.profession] ?? {
                                    color: "text-muted-foreground",
                                    bgColor: "bg-muted/10",
                                    progressColor: "bg-muted",
                                };
                                const isExpanded = expanded.has(prof.profession);
                                const hasSubProfessions = prof.subProfessions && prof.subProfessions.length > 0;

                                return (
                                    <div key={prof.profession}>
                                        <button className="group -mx-3 w-[calc(100%+1.5rem)] cursor-pointer rounded-lg px-3 py-2.5 text-left transition-colors hover:bg-muted/40" onClick={() => toggle(prof.profession)} type="button">
                                            <div className="flex items-center gap-2.5">
                                                <div className={`flex h-9 w-9 shrink-0 items-center justify-center rounded-md ${colors.bgColor}`}>
                                                    <ClassIcon className="opacity-90" profession={prof.profession} size={24} />
                                                </div>
                                                <div className="flex w-full flex-col gap-1.5">
                                                    <div className="flex items-center justify-between">
                                                        <div className="flex items-center gap-1.5">
                                                            <span className="font-medium text-sm">{prof.displayName}</span>
                                                            {hasSubProfessions && <ChevronDown className={cn("h-3.5 w-3.5 text-muted-foreground/50 transition-transform duration-200", isExpanded && "rotate-180")} />}
                                                        </div>
                                                        <div className="flex items-center gap-1">
                                                            <span className={`font-semibold text-sm tabular-nums ${colors.color}`}>{prof.owned}</span>
                                                            <span className="text-muted-foreground/60 text-sm">/ {prof.total}</span>
                                                        </div>
                                                    </div>
                                                    <div className="relative h-2 w-full overflow-hidden rounded-full bg-muted/50">
                                                        <div style={{ width: `${prof.percentage}%` }} className={`h-full rounded-full ${colors.progressColor}`} />
                                                    </div>
                                                </div>
                                            </div>
                                        </button>

                                        {hasSubProfessions && (
                                            <div className="grid transition-[grid-template-rows] duration-300 ease-in-out" style={{ gridTemplateRows: isExpanded ? "1fr" : "0fr" }}>
                                                <div className="min-h-0 overflow-hidden">
                                                    <div className="ml-4 grid py-1" style={{ gridTemplateColumns: "auto auto 1fr auto", "--sub-name-w": `${maxSubNameLen}ch` } as React.CSSProperties}>
                                                        {prof.subProfessions.map((sub, subIndex) => {
                                                            const isLast = subIndex === prof.subProfessions.length - 1;
                                                            return (
                                                                <div className="group/row relative col-span-4 grid grid-cols-subgrid items-center gap-x-2 rounded-md py-1.5 pr-2 transition-colors hover:bg-muted/20" key={sub.subProfessionId}>
                                                                    <div className={`absolute left-0 w-px bg-muted-foreground/20 ${isLast ? "top-0 h-1/2" : "top-0 h-full"}`} />
                                                                    <div className="absolute top-1/2 left-0 h-px w-6 bg-muted-foreground/20" />
                                                                    <div className="ml-6">
                                                                        <SubProfessionIcon className="shrink-0 opacity-70" size={16} subProfession={sub.subProfessionId} />
                                                                    </div>
                                                                    <span className="max-w-20 truncate text-foreground/70 text-xs sm:min-w-(--sub-name-w) sm:max-w-none">{sub.displayName.replace(/\s+\S+$/, "")}</span>
                                                                    <div className="relative h-1.5 min-w-0 overflow-hidden rounded-full bg-muted-foreground/15">
                                                                        <div style={{ width: `${sub.percentage}%` }} className={`h-full rounded-full ${colors.progressColor} opacity-70`} />
                                                                    </div>
                                                                    <span className={`w-10 shrink-0 text-right text-xs tabular-nums ${colors.color} opacity-80`}>
                                                                        {sub.owned}/{sub.total}
                                                                    </span>
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
                </div>
            </div>
            <div>
                <Card className="flex h-full flex-col border-border/50 bg-linear-to-br from-card/80 to-card/40 backdrop-blur-sm">
                    <CardHeader className="flex-none pb-2">
                        <CardTitle className="flex items-center gap-2 font-medium text-muted-foreground text-sm">
                            <Sparkles className="h-4 w-4" />
                            Skill Mastery
                        </CardTitle>
                    </CardHeader>
                    <CardContent className="flex flex-1 flex-col justify-center gap-4">
                        <div className="grid grid-cols-3 gap-3">
                            <Tooltip>
                                <TooltipTrigger>
                                    <div className="flex flex-col items-center gap-1 rounded-lg bg-muted/30 p-3">
                                        <span className="font-bold text-2xl text-rose-400 tabular-nums">{stats.masteries.m3Count}</span>
                                        <span className="text-muted-foreground text-xs">M3</span>
                                    </div>
                                </TooltipTrigger>
                                <TooltipContent sideOffset={5}>
                                    <p className="font-medium">M3 Operators</p>
                                    <p className="text-muted-foreground">Operators with at least one skill at Mastery 3</p>
                                </TooltipContent>
                            </Tooltip>

                            <Tooltip>
                                <TooltipTrigger>
                                    <div className="flex flex-col items-center gap-1 rounded-lg bg-muted/30 p-3">
                                        <span className="font-bold text-2xl text-amber-400 tabular-nums">{stats.masteries.m6Count}</span>
                                        <span className="text-muted-foreground text-xs">M6</span>
                                    </div>
                                </TooltipTrigger>
                                <TooltipContent sideOffset={5}>
                                    <p className="font-medium">M6 Operators</p>
                                    <p className="text-muted-foreground">Operators with 2 skills at Mastery 3</p>
                                </TooltipContent>
                            </Tooltip>

                            <Tooltip>
                                <TooltipTrigger>
                                    <div className="flex flex-col items-center gap-1 rounded-lg bg-muted/30 p-3">
                                        <span className="font-bold text-2xl text-violet-400 tabular-nums">{stats.masteries.m9Count}</span>
                                        <span className="text-muted-foreground text-xs">M9</span>
                                    </div>
                                </TooltipTrigger>
                                <TooltipContent sideOffset={5}>
                                    <p className="font-medium">M9 Operators</p>
                                    <p className="text-muted-foreground">Operators with all 3 skills at Mastery 3</p>
                                </TooltipContent>
                            </Tooltip>
                        </div>
                        <div className="space-y-2">
                            <div className="flex items-center justify-between text-xs">
                                <span className="text-muted-foreground">Total Mastery Levels</span>
                                <span className="font-medium tabular-nums">
                                    <span className="text-rose-400">{stats.masteries.totalMasteryLevels}</span>
                                    <span className="text-muted-foreground/60"> / {stats.masteries.maxPossibleMasteryLevels}</span>
                                </span>
                            </div>
                            <div className="relative h-1.5 w-full overflow-hidden rounded-full bg-muted/50">
                                <div style={{ width: `${stats.masteries.maxPossibleMasteryLevels > 0 ? (stats.masteries.totalMasteryLevels / stats.masteries.maxPossibleMasteryLevels) * 100 : 0}%` }} className="h-full rounded-full bg-rose-500" />
                            </div>
                            <p className="text-center text-[0.625rem] text-muted-foreground/70">{(stats.masteries.maxPossibleMasteryLevels > 0 ? (stats.masteries.totalMasteryLevels / stats.masteries.maxPossibleMasteryLevels) * 100 : 0).toFixed(1)}% of max possible (E2 operators only)</p>
                        </div>
                    </CardContent>
                </Card>
            </div>
            <div>
                <Card className="flex h-full flex-col border-border/50 bg-linear-to-br from-card/80 to-card/40 backdrop-blur-sm">
                    <CardContent className="flex flex-1 flex-col justify-center gap-5 pt-6">
                        {/* Module Stats */}
                        <div className="space-y-3">
                            <div className="flex items-center gap-2 text-muted-foreground">
                                <Layers className="h-4 w-4" />
                                <span className="font-medium text-sm">Modules</span>
                            </div>

                            <div className="grid grid-cols-2 gap-3">
                                <Tooltip>
                                    <TooltipTrigger>
                                        <div className="flex flex-col items-center gap-1 rounded-lg bg-muted/30 p-2.5">
                                            <div className="flex items-baseline gap-1">
                                                <span className="font-bold text-cyan-400 text-xl tabular-nums">{stats.modules.unlocked}</span>
                                                <span className="text-muted-foreground/60 text-sm">/ {stats.modules.totalAvailable}</span>
                                            </div>
                                            <span className="text-muted-foreground text-xs">Unlocked</span>
                                        </div>
                                    </TooltipTrigger>
                                    <TooltipContent sideOffset={5}>
                                        <p className="font-medium">Modules Unlocked</p>
                                        <p className="text-muted-foreground">
                                            {stats.modules.unlocked} of {stats.modules.totalAvailable} available modules
                                        </p>
                                    </TooltipContent>
                                </Tooltip>

                                <Tooltip>
                                    <TooltipTrigger>
                                        <div className="flex flex-col items-center gap-1 rounded-lg bg-muted/30 p-2.5">
                                            <span className="font-bold text-cyan-400 text-xl tabular-nums">{stats.modules.atMax}</span>
                                            <span className="text-muted-foreground text-xs">Max Level</span>
                                        </div>
                                    </TooltipTrigger>
                                    <TooltipContent sideOffset={5}>
                                        <p className="font-medium">Max Level Modules</p>
                                        <p className="text-muted-foreground">Modules upgraded to level 3</p>
                                    </TooltipContent>
                                </Tooltip>
                            </div>
                            <div>
                                <div className="relative h-1.5 w-full overflow-hidden rounded-full bg-muted/50">
                                    <div style={{ width: `${stats.modules.totalAvailable > 0 ? (stats.modules.unlocked / stats.modules.totalAvailable) * 100 : 0}%` }} className="h-full rounded-full bg-cyan-500" />
                                </div>
                                <p className="mt-1 text-center text-[0.625rem] text-muted-foreground/70">{(stats.modules.totalAvailable > 0 ? (stats.modules.unlocked / stats.modules.totalAvailable) * 100 : 0).toFixed(1)}% unlocked</p>
                            </div>
                        </div>
                        <div className="border-border/30 border-t" />
                        <div className="space-y-3">
                            <div className="flex items-center gap-2 text-muted-foreground">
                                <Palette className="h-4 w-4" />
                                <span className="font-medium text-sm">Skins</span>
                            </div>

                            <Tooltip>
                                <TooltipTrigger>
                                    <div className="flex flex-col items-center gap-1 rounded-lg bg-muted/30 p-2.5">
                                        <div className="flex items-baseline gap-1">
                                            <span className="font-bold text-pink-400 text-xl tabular-nums">{stats.skins.totalOwned}</span>
                                            <span className="text-muted-foreground/60 text-sm">/ {stats.skins.totalAvailable}</span>
                                        </div>
                                        <span className="text-muted-foreground text-xs">Skins Collected</span>
                                    </div>
                                </TooltipTrigger>
                                <TooltipContent sideOffset={5}>
                                    <p className="font-medium">Skins Collected</p>
                                    <p className="text-muted-foreground">
                                        {stats.skins.totalOwned} of {stats.skins.totalAvailable} non-default skins
                                    </p>
                                </TooltipContent>
                            </Tooltip>
                            <div>
                                <div className="relative h-1.5 w-full overflow-hidden rounded-full bg-muted/50">
                                    <div style={{ width: `${stats.skins.percentage}%` }} className="h-full rounded-full bg-pink-500" />
                                </div>
                                <p className="mt-1 text-center text-[0.625rem] text-muted-foreground/70">{stats.skins.percentage.toFixed(1)}% collected</p>
                            </div>
                        </div>
                    </CardContent>
                </Card>
            </div>
        </div>
    );
}
