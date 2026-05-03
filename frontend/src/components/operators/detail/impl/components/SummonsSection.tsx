import { useQuery } from "@tanstack/react-query";
import { ChevronDown, Sparkles } from "lucide-react";
import { useMemo, useState } from "react";
import { Badge } from "#/components/ui/badge";
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "#/components/ui/collapsible";
import { env } from "#/env";
import { rangesQueryOptions } from "#/lib/api/ranges";
import { cn } from "#/lib/utils";
import type { IDrone } from "#/types/operators";
import { descriptionToHtml } from "../description";
import { clampDronePhase, droneTalentBlackboard, getDroneAttributeStats } from "../helpers";
import { OperatorRange } from "./OperatorRange";

interface ISummonsSectionProps {
    drones: IDrone[];
    parentPhaseIndex: number;
    parentLevel: number;
}

const POSITION_LABEL: Record<string, string> = {
    MELEE: "Melee",
    RANGED: "Ranged",
    ALL: "Melee/Ranged",
    NONE: "—",
};

export function SummonsSection({ drones, parentPhaseIndex, parentLevel }: ISummonsSectionProps) {
    const [open, setOpen] = useState(true);
    const [activeId, setActiveId] = useState<string>(() => drones[0]?.id ?? "");

    const active = useMemo(() => drones.find((d) => d.id === activeId) ?? drones[0], [drones, activeId]);

    if (!active) return null;

    return (
        <Collapsible open={open} onOpenChange={setOpen}>
            <CollapsibleTrigger className="flex w-full items-center justify-between rounded-lg border border-border bg-secondary/30 px-4 py-3 transition-colors hover:bg-secondary/50">
                <span className="flex items-center gap-2">
                    <Sparkles className="h-4 w-4 text-primary" />
                    <span className="font-medium text-sm">Summons</span>
                    <Badge variant="outline" className="text-[10px]">
                        {drones.length}
                    </Badge>
                </span>
                <ChevronDown className={cn("h-4 w-4 transition-transform", open && "rotate-180")} />
            </CollapsibleTrigger>
            <CollapsibleContent>
                <div className="mt-3 space-y-3">
                    {drones.length > 1 && (
                        <div className="-mx-1 flex gap-1.5 overflow-x-auto px-1 pb-1">
                            {drones.map((d) => {
                                const isActive = d.id === active.id;
                                return (
                                    <button
                                        key={d.id ?? d.name}
                                        type="button"
                                        onClick={() => d.id && setActiveId(d.id)}
                                        className={cn("shrink-0 rounded-full border px-3 py-1 font-medium text-xs transition-colors", isActive ? "border-primary bg-primary/15 text-foreground" : "border-border bg-card text-muted-foreground hover:border-primary/40 hover:text-foreground")}
                                    >
                                        {d.name}
                                    </button>
                                );
                            })}
                        </div>
                    )}

                    <SummonCard drone={active} parentPhaseIndex={parentPhaseIndex} parentLevel={parentLevel} />
                </div>
            </CollapsibleContent>
        </Collapsible>
    );
}

interface ISummonCardProps {
    drone: IDrone;
    parentPhaseIndex: number;
    parentLevel: number;
}

function SummonCard({ drone, parentPhaseIndex, parentLevel }: ISummonCardProps) {
    const { data: ranges } = useQuery(rangesQueryOptions());

    const phaseIndex = clampDronePhase(drone, parentPhaseIndex);
    const phase = drone.phases[phaseIndex];
    const stats = useMemo(() => getDroneAttributeStats(drone, parentPhaseIndex, parentLevel), [drone, parentPhaseIndex, parentLevel]);
    const blackboard = useMemo(() => droneTalentBlackboard(drone), [drone]);
    const description = useMemo(() => descriptionToHtml(drone.description ?? "", blackboard), [drone.description, blackboard]);

    const range = phase?.rangeId ? ranges?.[phase.rangeId] : undefined;
    const avatarSrc = drone.id ? `${env.VITE_BACKEND_URL}/api/avatar/${drone.id}` : null;
    const positionLabel = POSITION_LABEL[drone.position] ?? drone.position;

    const fmt = (n: number | undefined) => (typeof n === "number" ? Math.round(n).toLocaleString() : "—");

    const leftStats = [
        { iconURL: "/stat-icons/HP.png", label: "Health", value: fmt(stats?.maxHp) },
        { iconURL: "/stat-icons/DEF.png", label: "Defense", value: fmt(stats?.def) },
        { iconURL: "/stat-icons/RES.png", label: "Arts Resistance", value: fmt(stats?.magicResistance) },
        { iconURL: "/stat-icons/RDP.png", label: "Redeploy Time", value: `${stats?.respawnTime ?? 0} sec` },
    ];
    const rightStats = [
        { iconURL: "/stat-icons/ATK.png", label: "Attack Power", value: fmt(stats?.atk) },
        { iconURL: "/stat-icons/ASPD.png", label: "Attack Interval", value: `${stats?.attackSpeed?.toFixed(2) ?? "0.00"} sec` },
        { iconURL: "/stat-icons/BLOCK.png", label: "Block", value: fmt(stats?.blockCnt) },
        { iconURL: "/stat-icons/COST.png", label: "DP Cost", value: fmt(stats?.cost) },
    ];

    const visibleTalents = (drone.talents ?? [])
        .map((t, idx) => {
            const c = t.candidates?.[t.candidates.length - 1];
            return c ? { key: `dt-${idx}`, name: c.name, description: c.description ?? "", blackboard: c.blackboard ?? [] } : null;
        })
        .filter((t): t is { key: string; name: string | null; description: string; blackboard: { key: string; value: number; valueStr: string | null }[] } => t !== null && (t.name !== null || t.description.length > 0));

    return (
        <div className="rounded-lg border border-border/60 bg-card/40 p-4">
            <div className="flex items-start gap-3">
                {avatarSrc && <img alt={drone.name} className="h-16 w-16 shrink-0 rounded-md border border-border/50 bg-secondary/30 object-contain" decoding="async" loading="lazy" src={avatarSrc} />}
                <div className="min-w-0 flex-1">
                    <div className="flex flex-wrap items-center gap-2">
                        <h4 className="truncate font-semibold text-foreground">{drone.name}</h4>
                        <Badge variant="outline" className="border-transparent bg-accent text-foreground text-xs">
                            {positionLabel}
                        </Badge>
                    </div>
                    {description && (
                        <p
                            className="mt-1 text-muted-foreground text-sm wrap-break-word"
                            // biome-ignore lint/security/noDangerouslySetInnerHtml: Sanitized in description-to-html
                            dangerouslySetInnerHTML={{ __html: description }}
                        />
                    )}
                </div>
            </div>

            <div className="mt-4 grid grid-cols-1 divide-y divide-border rounded-lg border border-border bg-card md:grid-cols-2 md:divide-x md:divide-y-0">
                {[leftStats, rightStats].map((column, colIdx) => (
                    // biome-ignore lint/suspicious/noArrayIndexKey: fixed two-column split, order is stable
                    <div key={colIdx} className="px-5 py-2">
                        {column.map(({ iconURL, label, value }) => (
                            <div key={label} className="flex items-center justify-between py-1.5">
                                <span className="flex items-center gap-2 text-muted-foreground">
                                    <img alt={label} className="h-4 w-4 object-contain icon-theme-aware" decoding="async" loading="lazy" src={iconURL} />
                                    <span className="text-xs">{label}</span>
                                </span>
                                <span className="font-semibold tabular-nums text-foreground text-sm">{value}</span>
                            </div>
                        ))}
                    </div>
                ))}
            </div>

            {range && (
                <div className="mt-4">
                    <h5 className="mb-2 font-medium text-foreground text-xs">Attack Range</h5>
                    <OperatorRange range={range} />
                </div>
            )}

            {visibleTalents.length > 0 && (
                <div className="mt-4 space-y-2">
                    <h5 className="font-medium text-foreground text-xs">Talents</h5>
                    {visibleTalents.map((t) => {
                        const html = descriptionToHtml(t.description, t.blackboard);
                        return (
                            <div key={t.key} className="rounded-md border border-border/40 bg-secondary/20 p-2">
                                {t.name && <span className="font-medium text-foreground text-xs">{t.name}: </span>}
                                <span
                                    className="text-muted-foreground text-xs"
                                    // biome-ignore lint/security/noDangerouslySetInnerHtml: Sanitized in description-to-html
                                    dangerouslySetInnerHTML={{ __html: html }}
                                />
                            </div>
                        );
                    })}
                </div>
            )}
        </div>
    );
}
