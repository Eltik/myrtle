import { useState } from "react";
import { OperatorAvatar } from "#/components/ui/operator-avatar";
import { Tooltip, TooltipContent, TooltipTrigger } from "#/components/ui/tooltip";
import type { IBaseAssignment, IRoomAssignment } from "#/lib/api/user";
import { cn } from "#/lib/utils";
import { assignmentTotals, compactNum, roomYield, roomYieldLabel, signedCompact } from "./baseYield";
import { TEXT_BADGE, TEXT_KICKER, TEXT_META } from "./shared";

const ROOM_LABELS: Record<string, string> = {
    MANUFACTURE: "Factory",
    TRADING: "Trading Post",
    CONTROL: "Control Center",
};

function roomLabel(t: string): string {
    return ROOM_LABELS[t] ?? t.charAt(0) + t.slice(1).toLowerCase();
}

/** The resource a room produces — so the comparison only lines up like-for-like. */
function resourceKey(room: IRoomAssignment): string {
    if (room.room_type === "TRADING") return "TRADING";
    if (room.room_type === "MANUFACTURE") {
        if (room.formula_type === "F_GOLD") return "GOLD";
        if (room.formula_type === "F_EXP") return "EXP";
        return "FACTORY";
    }
    return room.room_type;
}

const RESOURCE_GROUPS: { key: string; label: string }[] = [
    { key: "TRADING", label: "Trading Posts (LMD)" },
    { key: "GOLD", label: "Gold factories" },
    { key: "EXP", label: "EXP factories" },
    { key: "FACTORY", label: "Factories" },
];

interface ITarget {
    key: string;
    label: string;
    /** Left side: the player's base for this view. */
    cur: IBaseAssignment;
    /** Right side: the optimizer's recommendation for this view. */
    opt: IBaseAssignment;
}

interface IProps {
    /** Static current base (used for the "Peak" comparison). */
    current: IBaseAssignment;
    /** The player's planned preset Shift A / B (falls back to `current`). */
    currentShiftA: IBaseAssignment | null;
    currentShiftB: IBaseAssignment | null;
    optimal: IBaseAssignment | null;
    shiftA: IBaseAssignment | null;
    shiftB: IBaseAssignment | null;
    accent: string;
}

/**
 * Side-by-side comparison of the player's base against the optimizer's output.
 * Each tab pairs the right "current" side with its matching optimized side: the
 * peak vs the live base, and each rotation shift vs the player's own preset
 * shift. Rooms line up by slot.
 */
export function BaseComparison({ current, currentShiftA, currentShiftB, optimal, shiftA, shiftB, accent }: IProps) {
    const targets: ITarget[] = [optimal && { key: "peak", label: "Peak", cur: current, opt: optimal }, shiftA && { key: "a", label: "Shift A", cur: currentShiftA ?? current, opt: shiftA }, shiftB && { key: "b", label: "Shift B", cur: currentShiftB ?? current, opt: shiftB }].filter((t): t is ITarget => Boolean(t));

    const [tab, setTab] = useState(targets[0]?.key ?? "peak");
    const target = targets.find((t) => t.key === tab) ?? targets[0];
    if (!target) return null;

    const cur = assignmentTotals(target.cur);
    const opt = assignmentTotals(target.opt);

    // Group production rooms by RESOURCE (trading / gold / EXP) and pair them up
    // within each group, so the comparison is always like-for-like — gold→gold,
    // EXP→EXP — even when the optimizer re-assigns a factory's formula.
    const byEff = (a: IRoomAssignment, b: IRoomAssignment) => b.total_efficiency - a.total_efficiency;
    const groupRooms = (asn: IBaseAssignment, key: string) => asn.rooms.filter((r) => resourceKey(r) === key).sort(byEff);
    const groups = RESOURCE_GROUPS.map((g) => {
        const cur = groupRooms(target.cur, g.key);
        const opt = groupRooms(target.opt, g.key);
        const n = Math.max(cur.length, opt.length);
        const pairs = Array.from({ length: n }, (_, i) => ({ cur: cur[i], opt: opt[i] }));
        return { ...g, pairs };
    }).filter((g) => g.pairs.length > 0);

    return (
        <div className="flex flex-col gap-3">
            <div className="flex flex-wrap items-center justify-between gap-2">
                <p className={cn(TEXT_META, "text-muted-foreground")}>Your base vs the optimized plan, room by room. Highlighted operators are the swaps to make.</p>
                {targets.length > 1 && (
                    <div className="flex shrink-0 items-center gap-0.5 rounded-md border border-border/40 bg-muted/20 p-0.5">
                        {targets.map((t) => {
                            const active = t.key === tab;
                            return (
                                <button
                                    key={t.key}
                                    type="button"
                                    onClick={() => setTab(t.key)}
                                    aria-pressed={active}
                                    className={cn("rounded px-2 py-0.5 transition-colors", TEXT_KICKER, active ? "bg-background text-foreground shadow-sm" : "text-muted-foreground hover:text-foreground")}
                                    style={active ? { color: `color-mix(in oklch, ${accent} 70%, var(--foreground))` } : undefined}
                                >
                                    {t.label}
                                </button>
                            );
                        })}
                    </div>
                )}
            </div>

            {/* Headline totals */}
            <div className="grid grid-cols-3 gap-2">
                <TotalTile label="LMD / day" cur={cur.lmd} opt={opt.lmd} accent={accent} />
                <TotalTile label="EXP / day" cur={cur.exp} opt={opt.exp} accent={accent} />
                <TotalTile label="Total value / day" cur={cur.value} opt={opt.value} accent={accent} highlight />
            </div>

            {/* Per-resource comparison (gold→gold, EXP→EXP, trading→trading) */}
            <div className="grid grid-cols-[1fr_auto_1fr] gap-2 px-2">
                <span className={cn(TEXT_KICKER, "text-muted-foreground/70")}>Yours {target.key === "a" ? "(Shift A)" : target.key === "b" ? "(Shift B)" : "(now)"}</span>
                <span className="w-3" />
                <span className={cn(TEXT_KICKER, "text-right")} style={{ color: `color-mix(in oklch, ${accent} 65%, var(--foreground))` }}>
                    Optimal {target.label}
                </span>
            </div>
            <div className="flex flex-col gap-2.5">
                {groups.map((g) => (
                    <div key={g.key} className="flex flex-col gap-1.5">
                        <span className={cn(TEXT_BADGE, "px-1 text-muted-foreground/60")}>{g.label}</span>
                        {g.pairs.map((pair, i) => (
                            <CompareRow key={`${g.key}-${pair.cur?.slot_id ?? pair.opt?.slot_id ?? i}`} current={pair.cur} optimal={pair.opt} accent={accent} />
                        ))}
                    </div>
                ))}
            </div>
        </div>
    );
}

function TotalTile({ label, cur, opt, accent, highlight }: { label: string; cur: number; opt: number; accent: string; highlight?: boolean }) {
    const delta = opt - cur;
    return (
        <div className={cn("flex flex-col gap-0.5 rounded-md border px-2.5 py-2", highlight ? "border-border/60 bg-muted/25" : "border-border/40 bg-muted/10")}>
            <span className={cn(TEXT_KICKER, "text-muted-foreground")}>{label}</span>
            <div className="flex items-baseline gap-1.5">
                <span className="font-mono text-muted-foreground/70 text-xs tabular-nums">{compactNum(cur)}</span>
                <span className="text-muted-foreground/50">→</span>
                <span className="font-mono font-semibold text-sm tabular-nums" style={{ color: `color-mix(in oklch, ${accent} 70%, var(--foreground))` }}>
                    {compactNum(opt)}
                </span>
            </div>
            <span className={cn(TEXT_BADGE, delta > 0 ? "text-emerald-500/85" : delta < 0 ? "text-rose-500/80" : "text-muted-foreground/60")}>{signedCompact(delta)}</span>
        </div>
    );
}

function CompareRow({ current, optimal, accent }: { current?: IRoomAssignment; optimal?: IRoomAssignment; accent: string }) {
    const curIds = new Set((current?.operators ?? []).map((o) => o.operator_id));

    // The primary comparison is the per-day YIELD delta in the room's resource
    // (LMD / gold / EXP) — that's what actually matters, not the headline %.
    const curY = current ? roomYield(current) : null;
    const optY = optimal ? roomYield(optimal) : null;
    const unit = optY?.unit ?? curY?.unit ?? null;
    const yieldDelta = (optY?.amount ?? 0) - (curY?.amount ?? 0);
    // Efficiency % delta is kept as a secondary, smaller readout.
    const effDelta = current && optimal ? optimal.total_efficiency - current.total_efficiency : null;

    return (
        <div className="grid grid-cols-[1fr_auto_1fr] items-center gap-2 rounded-md border border-border/35 bg-muted/10 px-2 py-1.5">
            <RoomSide room={current} accent={accent} muted />
            <div className="flex min-w-[4.5rem] flex-col items-center gap-0.5">
                {unit ? (
                    <span className={cn("font-mono font-semibold text-xs tabular-nums", yieldDelta > 0.5 ? "text-emerald-500/90" : yieldDelta < -0.5 ? "text-rose-500/85" : "text-muted-foreground/60")}>
                        {signedCompact(yieldDelta)} {unit}
                    </span>
                ) : (
                    <span className="text-muted-foreground/50 text-xs">→</span>
                )}
                {effDelta !== null && (
                    <span className={cn(TEXT_BADGE, "text-muted-foreground/55")}>
                        {effDelta >= 0 ? "+" : "−"}
                        {Math.abs(effDelta).toFixed(0)}% eff
                    </span>
                )}
            </div>
            <RoomSide room={optimal} accent={accent} highlightIds={curIds} />
        </div>
    );
}

function RoomSide({ room, accent, muted, highlightIds }: { room?: IRoomAssignment; accent: string; muted?: boolean; highlightIds?: Set<string> }) {
    if (!room) {
        return <span className={cn(TEXT_BADGE, "text-muted-foreground/40")}>— none —</span>;
    }
    const yieldLabel = roomYieldLabel(room);
    return (
        <div className="flex min-w-0 flex-col gap-1">
            <div className="flex items-center justify-between gap-1.5">
                <span className={cn(TEXT_BADGE, "truncate text-muted-foreground")}>
                    {roomLabel(room.room_type)}
                    {room.formula_type ? ` · ${room.formula_type}` : ""}
                </span>
                <span className={cn("shrink-0 font-mono font-semibold tabular-nums", TEXT_BADGE)} style={{ color: `color-mix(in oklch, ${accent} 65%, var(--foreground))` }}>
                    +{room.total_efficiency.toFixed(0)}%
                </span>
            </div>
            <div className="flex flex-wrap gap-1">
                {room.operators.length === 0 && <span className={cn(TEXT_BADGE, "text-muted-foreground/50")}>empty</span>}
                {room.operators.map((op) => {
                    // In the optimized side, mark operators that aren't in the current room.
                    const isNew = highlightIds !== undefined && !highlightIds.has(op.operator_id);
                    return (
                        <Tooltip key={op.operator_id}>
                            <TooltipTrigger
                                render={
                                    <span className={cn("relative size-5 overflow-hidden rounded-sm border", isNew ? "border-2" : "border-border/40", muted && "opacity-70")} style={isNew ? { borderColor: `color-mix(in oklch, ${accent} 70%, transparent)` } : undefined}>
                                        <OperatorAvatar charId={op.operator_id} name={op.name} />
                                    </span>
                                }
                            />
                            <TooltipContent sideOffset={4}>
                                <p>
                                    {op.name}
                                    {isNew ? " · move here" : ""}
                                </p>
                            </TooltipContent>
                        </Tooltip>
                    );
                })}
            </div>
            {yieldLabel && <span className={cn(TEXT_BADGE, "text-muted-foreground/70")}>{yieldLabel}</span>}
        </div>
    );
}
