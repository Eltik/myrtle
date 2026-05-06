import { useQuery } from "@tanstack/react-query";
import { BarChart3, ChevronDown, Layers, Medal, Palette, Sparkles, Users } from "lucide-react";
import { useCallback, useMemo, useState } from "react";
import { eliteIcon } from "#/components/operators/detail/impl/assets";
import { ClassIcon, SubProfessionIcon } from "#/components/operators/list/impl/components/Icons";
import { RARITY_COLORS } from "#/components/operators/list/impl/constants";
import { Tooltip, TooltipContent, TooltipTrigger } from "#/components/ui/tooltip";
import { skinsQueryOptions } from "#/lib/api/skins";
import type { IRosterEntry } from "#/lib/api/user";
import { cn, rarityToNumber } from "#/lib/utils";
import type { IOperatorListItem } from "#/types/operators";
import { MAX_LEVEL_BY_RARITY, ownedAvatar, parseOperatorName, specializedIcon } from "../Roster/helpers.card";
import { computeUserStats } from "./helpers";

interface IStatsViewProps {
    roster: IRosterEntry[];
    operatorsStatic: IOperatorListItem[];
}

// Each section owns a distinct hue so cards read as separate, but lightness and
// chroma are kept close so the page stays harmonious. Add new sections by
// extending PALETTE — every card pulls from here, nothing is hard-coded.
const PALETTE = {
    collection: "var(--primary)", // terracotta — brand anchor
    elite: "oklch(0.74 0.17 75)", // amber — E2 gold pip association
    top: "var(--primary)", // brand for the top-operator ribbon

    // Class breakdown: 8 hues evenly spaced across the wheel at matched
    // L≈0.66 / C≈0.18 so the bars stay distinguishable without one shouting.
    classes: {
        PIONEER: "oklch(0.70 0.14 200)", // cyan
        WARRIOR: "oklch(0.62 0.22 25)", // terracotta (brand-adjacent)
        TANK: "oklch(0.74 0.16 75)", // amber
        SNIPER: "oklch(0.62 0.20 255)", // blue
        CASTER: "oklch(0.62 0.22 295)", // violet
        SUPPORT: "oklch(0.70 0.16 145)", // emerald
        MEDIC: "oklch(0.78 0.16 110)", // lime
        SPECIAL: "oklch(0.65 0.22 340)", // magenta
    } as Record<string, string>,

    // Skill Mastery: card is violet; tier tiles trace a progression
    // emerald → blue → gold so M3 / M6 / M9 each read on their own.
    mastery: {
        accent: "oklch(0.62 0.21 295)", // violet
        m3: "oklch(0.70 0.17 145)", // emerald — first tier
        m6: "oklch(0.65 0.18 230)", // blue — middle tier
        m9: "oklch(0.74 0.17 75)", // gold — peak
    },

    // Modules + Skins share a card; the top accent line uses modules' cyan,
    // and skins gets its own magenta accent inside so the two sections don't
    // blur together.
    modules: {
        accent: "oklch(0.66 0.14 200)", // cyan
        unlocked: "oklch(0.66 0.14 200)", // cyan
        max: "oklch(0.72 0.15 175)", // teal
    },
    skins: "oklch(0.65 0.22 340)", // magenta
} as const;

const EXCLUDED_MODULE_KEYS = ["uniequip_000", "uniequip_001"];

function operatorScore(entry: IRosterEntry, rarity: number): number {
    const maxLevel = MAX_LEVEL_BY_RARITY[rarity]?.[entry.elite] ?? 90;
    const mods = entry.modules.filter((m) => !EXCLUDED_MODULE_KEYS.some((k) => m.id.startsWith(k)) && !m.locked && m.level > 0);
    return entry.elite * 300 + (entry.level / maxLevel) * 100 + entry.masteries.reduce((s, m) => s + m.mastery * 15, 0) + mods.length * 25 + mods.filter((m) => m.level === 3).length * 15;
}

function barGradient(color: string) {
    return `linear-gradient(to right, color-mix(in oklch, ${color} 55%, transparent), ${color})`;
}

function lineGradient(color: string | readonly string[]) {
    if (typeof color === "string") {
        return `linear-gradient(to right, transparent, color-mix(in oklch, ${color} 70%, transparent), transparent)`;
    }
    // Multi-stop: fade in, walk through each color, fade out — used by the
    // class card to preview its per-class palette.
    const stops = color.map((c) => `color-mix(in oklch, ${c} 70%, transparent)`).join(", ");
    return `linear-gradient(to right, transparent, ${stops}, transparent)`;
}

function StatCard({ children, color, className }: { children: React.ReactNode; color: string | readonly string[]; className?: string }) {
    return (
        <div className={cn("relative overflow-hidden rounded-xl sm:rounded-2xl border border-border bg-card", "shadow-[0_1px_2px_rgb(0_0_0/0.04)] transition-colors duration-200 hover:border-foreground/15", className)}>
            <div aria-hidden className="pointer-events-none absolute inset-x-0 top-0 z-10 h-px" style={{ background: lineGradient(color) }} />
            {children}
        </div>
    );
}

function Kicker({ icon: Icon, label }: { icon: React.ElementType; label: string }) {
    return (
        <div className="flex items-center gap-2">
            <Icon className="h-3.5 w-3.5 shrink-0 text-muted-foreground/60" />
            <span className="font-mono font-semibold text-[10.5px] uppercase tracking-[0.12em] text-muted-foreground">{label}</span>
        </div>
    );
}

function Bar({ pct, color, thin, dim }: { pct: number; color: string; thin?: boolean; dim?: boolean }) {
    return (
        <div className={cn("w-full overflow-hidden rounded-full bg-muted/50", thin ? "h-1" : "h-1.5")}>
            <div
                className="h-full rounded-full transition-[width] duration-700 ease-out"
                style={{
                    width: `${Math.min(pct, 100)}%`,
                    background: barGradient(color),
                    opacity: dim ? 0.5 : 1,
                }}
            />
        </div>
    );
}

function Tile({ value, sub, color, tooltip }: { value: React.ReactNode; sub: string; color: string; tooltip?: string }) {
    const inner = (
        <div className="flex cursor-default flex-col items-center gap-1.5 rounded-lg border border-border/40 bg-muted/20 p-3 text-center transition-colors hover:bg-muted/35">
            <span className="font-bold tabular-nums leading-none" style={{ fontSize: "1.375rem", letterSpacing: "-0.02em", color }}>
                {value}
            </span>
            <span className="font-mono font-semibold text-[10.5px] uppercase tracking-[0.12em] text-muted-foreground">{sub}</span>
        </div>
    );
    if (!tooltip) return inner;
    return (
        <Tooltip>
            <TooltipTrigger render={inner} />
            <TooltipContent sideOffset={5}>
                <p>{tooltip}</p>
            </TooltipContent>
        </Tooltip>
    );
}

function MetricRow({ label, value, pct, color }: { label: string; value: React.ReactNode; pct: number; color: string }) {
    return (
        <div className="space-y-2">
            <div className="flex items-center justify-between">
                <span className="font-mono font-semibold text-[10.5px] uppercase tracking-[0.12em] text-muted-foreground">{label}</span>
                <span className="font-mono text-[10.5px] tabular-nums text-muted-foreground">{value}</span>
            </div>
            <Bar color={color} pct={pct} />
        </div>
    );
}

function TopOperators({ roster, opMap }: { roster: IRosterEntry[]; opMap: Map<string, IOperatorListItem> }) {
    const top = useMemo(() => {
        return roster
            .map((entry) => {
                const op = opMap.get(entry.operator_id);
                if (!op || op.isNotObtainable) return null;
                const rarity = rarityToNumber(op.rarity);
                return { entry, op, rarity, score: operatorScore(entry, rarity) };
            })
            .filter(Boolean)
            .sort((a, b) => b?.score - a?.score)
            .slice(0, 8) as { entry: IRosterEntry; op: IOperatorListItem; rarity: number; score: number }[];
    }, [roster, opMap]);

    if (top.length === 0) return null;

    return (
        <StatCard color={PALETTE.top} className="sm:col-span-2">
            <div className="p-4 pb-3 sm:p-5 sm:pb-3">
                <Kicker icon={Medal} label="Top Operators" />
            </div>
            <div className="grid grid-cols-2 gap-px bg-border sm:grid-cols-4">
                {top.map(({ entry, op }, i) => {
                    const rarity = rarityToNumber(op.rarity);
                    const rarityColor = RARITY_COLORS[rarity] ?? "#ffffff";
                    const { displayName } = parseOperatorName(op.name);
                    const avatarUrl = ownedAvatar(entry.operator_id, entry.skin_id);
                    const hasMasteries = entry.masteries.some((m) => m.mastery > 0);

                    return (
                        <div className="relative flex flex-col gap-2.5 bg-card p-3 sm:p-4" key={entry.operator_id}>
                            <div aria-hidden className="absolute inset-x-0 top-0 h-0.5" style={{ background: rarityColor, opacity: 0.55 }} />
                            <div className="absolute top-2.5 right-2.5 font-mono text-[10px] font-bold tabular-nums text-muted-foreground/35">#{i + 1}</div>

                            <div className="flex items-center gap-2.5 pt-0.5">
                                <div className="relative shrink-0">
                                    <img alt={displayName} className="h-10 w-10 rounded-lg object-cover" src={avatarUrl} style={{ background: `${rarityColor}18` }} />
                                    <img alt={`E${entry.elite}`} className="absolute -right-1.5 -bottom-1.5 h-4 w-4" src={eliteIcon(entry.elite)} />
                                </div>
                                <div className="min-w-0 flex-1">
                                    <p className="truncate font-semibold text-sm leading-tight" style={{ maxWidth: "9rem" }}>
                                        {displayName}
                                    </p>
                                    <p className="font-mono text-[10.5px] text-muted-foreground/70">Lv {entry.level}</p>
                                </div>
                            </div>

                            {hasMasteries && <div className="flex gap-1">{entry.masteries.map((m) => (m.mastery > 0 ? <img alt={`M${m.mastery}`} className="h-4 w-4 opacity-90" key={m.index} src={specializedIcon(m.mastery)} /> : <div className="h-4 w-4 rounded-sm bg-muted/40" key={m.index} />))}</div>}
                        </div>
                    );
                })}
            </div>
        </StatCard>
    );
}

export function StatsTab({ roster, operatorsStatic }: IStatsViewProps) {
    const { data: skinData } = useQuery(skinsQueryOptions());
    const stats = useMemo(() => computeUserStats(roster, operatorsStatic, skinData?.charSkins), [roster, operatorsStatic, skinData]);

    const opMap = useMemo(() => {
        const m = new Map<string, IOperatorListItem>();
        for (const op of operatorsStatic) if (op.id) m.set(op.id, op);
        return m;
    }, [operatorsStatic]);

    const maxSubNameLen = useMemo(() => {
        let max = 0;
        for (const prof of stats.professions)
            for (const sub of prof.subProfessions) {
                const n = sub.displayName.replace(/\s+\S+$/, "").length;
                if (n > max) max = n;
            }
        return max;
    }, [stats.professions]);

    const classBreakdownColors = useMemo(() => stats.professions.map((p) => PALETTE.classes[p.profession] ?? PALETTE.collection), [stats.professions]);

    const [expanded, setExpanded] = useState<Set<string>>(new Set());
    const toggle = useCallback((p: string) => {
        setExpanded((prev) => {
            const next = new Set(prev);
            if (next.has(p)) next.delete(p);
            else next.add(p);
            return next;
        });
    }, []);

    if (!skinData?.charSkins) return <></>;

    const eliteRows = [
        { key: "e2", label: "Elite 2", count: stats.eliteBreakdown.e2 },
        { key: "e1", label: "Elite 1", count: stats.eliteBreakdown.e1 },
        { key: "e0", label: "Elite 0", count: stats.eliteBreakdown.e0 },
    ];

    const masteryPct = stats.masteries.maxPossibleMasteryLevels > 0 ? (stats.masteries.totalMasteryLevels / stats.masteries.maxPossibleMasteryLevels) * 100 : 0;

    const moduleUnlockPct = stats.modules.totalAvailable > 0 ? (stats.modules.unlocked / stats.modules.totalAvailable) * 100 : 0;

    const PAD = "p-4 sm:p-5";

    return (
        <div className="grid gap-3 pb-8 sm:grid-cols-2">
            <StatCard color={PALETTE.collection}>
                <div className={cn("flex h-full flex-col gap-5", PAD)}>
                    <Kicker icon={Users} label="Operator Collection" />
                    <div className="flex flex-1 flex-col justify-between gap-5">
                        <div className="flex flex-col items-center gap-1.5 py-1">
                            <div className="flex items-baseline gap-1.5">
                                <span
                                    className="font-bold tabular-nums leading-none"
                                    style={{
                                        fontSize: "clamp(2.25rem, 3vw + 1rem, 3rem)",
                                        letterSpacing: "-0.03em",
                                        color: PALETTE.collection,
                                    }}
                                >
                                    {stats.totalOwned}
                                </span>
                                <span className="font-mono text-lg font-medium tabular-nums text-muted-foreground/50">/ {stats.totalAvailable}</span>
                            </div>
                            <span className="font-mono font-semibold text-[10.5px] uppercase tracking-[0.12em] text-muted-foreground">operators collected</span>
                        </div>
                        <MetricRow color={PALETTE.collection} label="Completion" pct={stats.collectionPercentage} value={`${Math.round((stats.collectionPercentage + Number.EPSILON) * 100) / 100}%`} />
                    </div>
                </div>
            </StatCard>

            <StatCard color={PALETTE.elite}>
                <div className={cn("flex h-full flex-col gap-5", PAD)}>
                    <Kicker icon={BarChart3} label="Elite Promotion" />
                    <div className="flex flex-1 flex-col justify-center gap-4">
                        {eliteRows.map(({ key, label, count }, i) => {
                            const pct = stats.eliteBreakdown.total > 0 ? (count / stats.eliteBreakdown.total) * 100 : 0;
                            return (
                                <div className="space-y-1.5" key={key}>
                                    <div className="flex items-center justify-between">
                                        <span className="font-mono font-semibold text-[10.5px] uppercase tracking-[0.12em] text-muted-foreground">{label}</span>
                                        <div className="flex items-baseline gap-1.5">
                                            <span className="font-bold tabular-nums text-sm" style={i === 0 ? { color: PALETTE.elite } : undefined}>
                                                {count}
                                            </span>
                                            <span className="font-mono text-[10px] tabular-nums text-muted-foreground/50">{pct.toFixed(0)}%</span>
                                        </div>
                                    </div>
                                    <Bar color={PALETTE.elite} dim={i > 0} pct={pct} />
                                </div>
                            );
                        })}
                    </div>
                </div>
            </StatCard>

            <StatCard className="sm:col-span-2" color={classBreakdownColors}>
                <div className={cn(PAD, "pb-2")}>
                    <Kicker icon={BarChart3} label="Class Breakdown" />
                </div>
                <div className="flex flex-col px-4 pb-4 sm:px-5 sm:pb-5">
                    {stats.professions.map((prof) => {
                        const isExpanded = expanded.has(prof.profession);
                        const hasSubs = prof.subProfessions.length > 0;
                        const classColor = PALETTE.classes[prof.profession] ?? PALETTE.collection;

                        return (
                            <div key={prof.profession}>
                                <button className="group -mx-3 w-[calc(100%+1.5rem)] cursor-pointer rounded-lg px-3 py-2.5 text-left transition-colors hover:bg-muted/40" onClick={() => toggle(prof.profession)} type="button">
                                    <div className="flex items-center gap-3">
                                        <div className="flex h-9 w-9 shrink-0 items-center justify-center rounded-xl" style={{ background: `color-mix(in oklch, ${classColor} 14%, transparent)` }}>
                                            <ClassIcon className="opacity-80" profession={prof.profession} size={22} />
                                        </div>
                                        <div className="flex w-full min-w-0 flex-col gap-1.5">
                                            <div className="flex items-center justify-between">
                                                <div className="flex items-center gap-1.5">
                                                    <span className="font-semibold text-sm">{prof.displayName}</span>
                                                    {hasSubs && <ChevronDown className={cn("h-3.5 w-3.5 text-muted-foreground/40 transition-transform duration-200", isExpanded && "rotate-180")} />}
                                                </div>
                                                <div className="flex items-baseline gap-1">
                                                    <span className="font-bold tabular-nums text-sm">{prof.owned}</span>
                                                    <span className="font-mono text-[10.5px] text-muted-foreground/50">/ {prof.total}</span>
                                                </div>
                                            </div>
                                            <Bar color={classColor} pct={prof.percentage} />
                                        </div>
                                    </div>
                                </button>

                                {hasSubs && (
                                    <div className="grid transition-[grid-template-rows] duration-300 ease-in-out" style={{ gridTemplateRows: isExpanded ? "1fr" : "0fr" }}>
                                        <div className="min-h-0 overflow-hidden">
                                            <div
                                                className="ml-4 grid py-1"
                                                style={{
                                                    gridTemplateColumns: "auto auto 1fr auto",
                                                    ["--sub-name-w" as string]: `${maxSubNameLen}ch`,
                                                }}
                                            >
                                                {prof.subProfessions.map((sub, idx) => {
                                                    const isLast = idx === prof.subProfessions.length - 1;
                                                    return (
                                                        <div className="group/row relative col-span-4 grid grid-cols-subgrid items-center gap-x-2 rounded-md py-1.5 pr-2 transition-colors hover:bg-muted/20" key={sub.subProfessionId}>
                                                            <div className={cn("absolute left-0 w-px bg-muted-foreground/20", isLast ? "top-0 h-1/2" : "top-0 h-full")} />
                                                            <div className="absolute top-1/2 left-0 h-px w-6 bg-muted-foreground/20" />
                                                            <div className="ml-6">
                                                                <SubProfessionIcon className="shrink-0 opacity-60" size={15} subProfession={sub.subProfessionId} />
                                                            </div>
                                                            <span className="max-w-20 truncate font-mono text-[10.5px] text-muted-foreground/70 sm:max-w-none" style={{ minWidth: "var(--sub-name-w)" }}>
                                                                {sub.displayName.replace(/\s+\S+$/, "")}
                                                            </span>
                                                            <Bar color={classColor} dim pct={sub.percentage} thin />
                                                            <span className="w-10 shrink-0 text-right font-mono text-[10.5px] tabular-nums text-muted-foreground/70">
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
                </div>
            </StatCard>

            <StatCard color={PALETTE.mastery.accent}>
                <div className={cn("flex h-full flex-col gap-5", PAD)}>
                    <Kicker icon={Sparkles} label="Skill Mastery" />
                    <div className="flex flex-1 flex-col justify-center gap-5">
                        <div className="grid grid-cols-3 gap-2">
                            <Tile color={PALETTE.mastery.m3} sub="M3" tooltip="Operators with at least one Mastery 3 skill" value={stats.masteries.m3Count} />
                            <Tile color={PALETTE.mastery.m6} sub="M6" tooltip="Operators with 2 skills at Mastery 3" value={stats.masteries.m6Count} />
                            <Tile color={PALETTE.mastery.m9} sub="M9" tooltip="Operators with all 3 skills at Mastery 3" value={stats.masteries.m9Count} />
                        </div>
                        <div className="space-y-2">
                            <div className="flex items-center justify-between">
                                <span className="font-mono font-semibold text-[10.5px] uppercase tracking-[0.12em] text-muted-foreground">Total Mastery Levels</span>
                                <span className="font-mono text-[10.5px] tabular-nums text-muted-foreground">
                                    <span className="font-semibold text-foreground">{stats.masteries.totalMasteryLevels}</span>
                                    <span className="text-muted-foreground/50"> / {stats.masteries.maxPossibleMasteryLevels}</span>
                                </span>
                            </div>
                            <Bar color={PALETTE.mastery.accent} pct={masteryPct} />
                            <p className="text-center font-mono text-[10px] text-muted-foreground/50">{masteryPct.toFixed(1)}% of max possible (E2 only)</p>
                        </div>
                    </div>
                </div>
            </StatCard>

            <StatCard color={PALETTE.modules.accent}>
                <div className={cn("flex h-full flex-col gap-5", PAD)}>
                    <div className="space-y-4">
                        <Kicker icon={Layers} label="Modules" />
                        <div className="grid grid-cols-2 gap-2">
                            <Tile
                                color={PALETTE.modules.unlocked}
                                sub="Unlocked"
                                tooltip={`${stats.modules.unlocked} of ${stats.modules.totalAvailable} available modules unlocked`}
                                value={
                                    <span>
                                        {stats.modules.unlocked}
                                        <span className="ml-1 text-sm font-medium text-muted-foreground/50">/ {stats.modules.totalAvailable}</span>
                                    </span>
                                }
                            />
                            <Tile color={PALETTE.modules.max} sub="Max Lv" tooltip="Modules upgraded to level 3" value={stats.modules.atMax} />
                        </div>
                        <MetricRow color={PALETTE.modules.accent} label="Unlock Rate" pct={moduleUnlockPct} value={`${moduleUnlockPct.toFixed(1)}%`} />
                    </div>

                    <div className="border-t border-border/60" />

                    <div className="space-y-4">
                        <Kicker icon={Palette} label="Skins" />
                        <Tile
                            color={PALETTE.skins}
                            sub="Skins Collected"
                            tooltip={`${stats.skins.totalOwned} of ${stats.skins.totalAvailable} non-default skins collected`}
                            value={
                                <span>
                                    {stats.skins.totalOwned}
                                    <span className="ml-1 text-sm font-medium text-muted-foreground/50">/ {stats.skins.totalAvailable}</span>
                                </span>
                            }
                        />
                        <MetricRow color={PALETTE.skins} label="Collected" pct={stats.skins.percentage} value={`${stats.skins.percentage.toFixed(1)}%`} />
                    </div>
                </div>
            </StatCard>
            <TopOperators opMap={opMap} roster={roster} />
        </div>
    );
}
