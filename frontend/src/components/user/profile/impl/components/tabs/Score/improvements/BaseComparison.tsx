import { OperatorAvatar } from "#/components/ui/operator-avatar";
import { Tooltip, TooltipContent, TooltipTrigger } from "#/components/ui/tooltip";
import type { IBaseAssignment, IRoomAssignment } from "#/lib/api/user";
import { cn } from "#/lib/utils";
import { assignmentTotals, compactNum, roomYield, roomYieldLabel, signedCompact } from "./baseYield";
import { roomAccent, roomLabel } from "./roomColors";
import { TEXT_BADGE, TEXT_KICKER, TEXT_META } from "./shared";

/** The resource a room produces - so the comparison only lines up like-for-like. */
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

interface IProps {
    /** The player's current base. */
    current: IBaseAssignment;
    /** The optimizer's peak (main) staffing. */
    optimal: IBaseAssignment | null;
    accent: string;
}

/**
 * Side-by-side comparison of the player's current base against the optimizer's
 * peak staffing - Control Center and room by room. Rooms line up by resource.
 */
export function BaseComparison({ current, optimal, accent }: IProps) {
    if (!optimal) return null;

    const cur = assignmentTotals(current);
    const opt = assignmentTotals(optimal);

    // The Control Center has no resource yield of its own, but its operators drive
    // a global production buff applied to every factory and trading post, so the
    // swaps belong in the comparison too.
    const curCC = current.rooms.find((r) => r.room_type === "CONTROL");
    const optCC = optimal.rooms.find((r) => r.room_type === "CONTROL");

    // Group production rooms by RESOURCE (trading / gold / EXP) and pair them up
    // within each group, so the comparison is always like-for-like - gold→gold,
    // EXP→EXP - even when the optimizer re-assigns a factory's formula.
    const byEff = (a: IRoomAssignment, b: IRoomAssignment) => b.total_efficiency - a.total_efficiency;
    const groupRooms = (asn: IBaseAssignment, key: string) => asn.rooms.filter((r) => resourceKey(r) === key).sort(byEff);
    const groups = RESOURCE_GROUPS.map((g) => {
        const curRooms = groupRooms(current, g.key);
        const optRooms = groupRooms(optimal, g.key);
        const n = Math.max(curRooms.length, optRooms.length);
        const pairs = Array.from({ length: n }, (_, i) => ({ cur: curRooms[i], opt: optRooms[i] }));
        return { ...g, pairs };
    }).filter((g) => g.pairs.length > 0);

    return (
        <div className="flex flex-col gap-3">
            <p className={cn(TEXT_META, "text-muted-foreground")}>Your base vs the optimized peak, Control Center and room by room. Highlighted operators are the swaps to make.</p>

            {/* Headline totals */}
            <div className="grid grid-cols-3 gap-2">
                <TotalTile label="LMD / day" cur={cur.lmd} opt={opt.lmd} accent={accent} />
                <TotalTile label="EXP / day" cur={cur.exp} opt={opt.exp} accent={accent} />
                <TotalTile label="Total value / day" cur={cur.value} opt={opt.value} accent={accent} highlight />
            </div>

            {/* Per-resource comparison (gold→gold, EXP→EXP, trading→trading) */}
            <div className="grid grid-cols-[1fr_auto_1fr] gap-2 px-2">
                <span className={cn(TEXT_KICKER, "text-muted-foreground/70")}>Yours (now)</span>
                <span className="w-3" />
                <span className={cn(TEXT_KICKER, "text-right")} style={{ color: `color-mix(in oklch, ${accent} 65%, var(--foreground))` }}>
                    Optimal peak
                </span>
            </div>
            <div className="flex flex-col gap-2.5">
                {(curCC || optCC) && (
                    <div className="flex flex-col gap-1.5">
                        <GroupLabel roomType="CONTROL" label="Control Center (global buff)" />
                        <CompareRow current={curCC} optimal={optCC} accent={accent} />
                    </div>
                )}
                {groups.map((g) => (
                    <div key={g.key} className="flex flex-col gap-1.5">
                        <GroupLabel roomType={g.key === "TRADING" ? "TRADING" : "MANUFACTURE"} label={g.label} />
                        {g.pairs.map((pair, i) => (
                            <CompareRow key={`${g.key}-${pair.cur?.slot_id ?? pair.opt?.slot_id ?? i}`} current={pair.cur} optimal={pair.opt} accent={accent} />
                        ))}
                    </div>
                ))}
            </div>
        </div>
    );
}

/** A resource-group heading tinted with the building's in-game colour. */
function GroupLabel({ roomType, label }: { roomType: string; label: string }) {
    const a = roomAccent(roomType);
    return (
        <span className={cn(TEXT_BADGE, "flex items-center gap-1.5 px-1")} style={{ color: a.text }}>
            <span className="size-1.5 rounded-[2px]" style={{ background: a.color }} />
            {label}
        </span>
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

    // The Control Center produces no resource yield - its metric is the global
    // production buff it grants (its `total_efficiency`), labelled accordingly.
    const isControl = (current ?? optimal)?.room_type === "CONTROL";

    // The primary comparison is the per-day YIELD delta in the room's resource
    // (LMD / gold / EXP) - that's what actually matters, not the headline %.
    const curY = current ? roomYield(current) : null;
    const optY = optimal ? roomYield(optimal) : null;
    const unit = optY?.unit ?? curY?.unit ?? null;
    const yieldDelta = (optY?.amount ?? 0) - (curY?.amount ?? 0);
    // % delta (order speed for production rooms, global buff for the CC).
    const pctDelta = current && optimal ? optimal.total_efficiency - current.total_efficiency : null;
    const pctLabel = isControl ? "global" : "eff";
    const pctColor = (d: number) => (d > 0.05 ? "text-emerald-500/90" : d < -0.05 ? "text-rose-500/85" : "text-muted-foreground/60");

    return (
        <div className="grid grid-cols-[1fr_auto_1fr] items-center gap-2 rounded-md border border-border/35 bg-muted/10 px-2 py-1.5">
            <RoomSide room={current} accent={accent} muted />
            <div className="flex min-w-18 flex-col items-center gap-0.5">
                {unit ? (
                    // Yield-bearing room: yield delta primary, % delta secondary.
                    <>
                        <span className={cn("font-mono font-semibold text-xs tabular-nums", pctColor(yieldDelta))}>
                            {signedCompact(yieldDelta)} {unit}
                        </span>
                        {pctDelta !== null && (
                            <span className={cn(TEXT_BADGE, "text-muted-foreground/55")}>
                                {pctDelta >= 0 ? "+" : "−"}
                                {Math.abs(pctDelta).toFixed(0)}% {pctLabel}
                            </span>
                        )}
                    </>
                ) : pctDelta !== null ? (
                    // No yield (Control Center): the % buff delta is the primary metric.
                    <span className={cn("font-mono font-semibold text-xs tabular-nums", pctColor(pctDelta))}>
                        {pctDelta >= 0 ? "+" : "−"}
                        {Math.abs(pctDelta).toFixed(0)}% {pctLabel}
                    </span>
                ) : (
                    <span className="text-muted-foreground/50 text-xs">→</span>
                )}
            </div>
            <RoomSide room={optimal} accent={accent} highlightIds={curIds} />
        </div>
    );
}

function RoomSide({ room, accent, muted, highlightIds }: { room?: IRoomAssignment; accent: string; muted?: boolean; highlightIds?: Set<string> }) {
    if (!room) {
        return <span className={cn(TEXT_BADGE, "text-muted-foreground/40")}>- none -</span>;
    }
    const yieldLabel = roomYieldLabel(room);
    const a = roomAccent(room.room_type);
    return (
        <div className="flex min-w-0 flex-col gap-1">
            <div className="flex items-center justify-between gap-1.5">
                <span className={cn(TEXT_BADGE, "flex items-center gap-1 truncate")} style={{ color: a.text }}>
                    <span className="size-1.5 shrink-0 rounded-[2px]" style={{ background: a.color }} />
                    {roomLabel(room.room_type)}
                    {room.formula_type ? ` · ${room.formula_type}` : ""}
                </span>
                <span className={cn("shrink-0 font-mono font-semibold tabular-nums", TEXT_BADGE)} style={{ color: a.strong }}>
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
