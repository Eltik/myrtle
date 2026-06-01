import { Dialog, DialogPanel, DialogPopup, DialogTitle, DialogTrigger } from "#/components/ui/dialog";
import { OperatorAvatar } from "#/components/ui/operator-avatar";
import { Tooltip, TooltipContent, TooltipTrigger } from "#/components/ui/tooltip";
import type { IAssignedOperator, IBaseAssignment, IBaseImprovements, IImprovementsResponse, IRoomAssignment, IRoomRotation, IRotation, IRotationSet } from "#/lib/api/user";
import { cn } from "#/lib/utils";
import { BaseComparison } from "./BaseComparison";
import { compactNum, roomYieldLabel, signedCompact } from "./baseYield";
import { EmptyHint, PANEL_PADDING, Pill, SectionHeader, TEXT_BADGE, TEXT_BODY, TEXT_KICKER, TEXT_META } from "./shared";

interface IProps {
    improvements: IImprovementsResponse;
    accent: string;
}

const ROOM_LABELS: Record<string, string> = {
    MANUFACTURE: "Factory",
    TRADING: "Trading Post",
    POWER: "Power Plant",
    DORMITORY: "Dormitory",
    WORKSHOP: "Workshop",
    MEETING: "Reception",
    HIRE: "Office",
    TRAINING: "Training",
    CONTROL: "Control Center",
};

function roomLabel(t: string): string {
    return ROOM_LABELS[t] ?? t.charAt(0) + t.slice(1).toLowerCase();
}

/** A "/day" suffix that clarifies the per-day LMD/gold/EXP figures are an AVERAGE
 *  over the staggered rotation cycle (~36h), not a constant peak rate. */
function PerDay({ className }: { className?: string }) {
    return (
        <Tooltip>
            <TooltipTrigger render={<span className={cn("cursor-help underline decoration-dotted underline-offset-2", className)}>/day</span>} />
            <TooltipContent sideOffset={4}>
                <p className="max-w-xs">Averaged over a ~36h staggered-rotation cycle - real output dips a little while a backup covers a resting operator, so this is the sustained daily average, not a constant peak.</p>
            </TooltipContent>
        </Tooltip>
    );
}

/** "X LMD/day + Y EXP/day" summary for an assignment's realized yield. */
function YieldSummary({ assignment }: { assignment: IBaseAssignment }) {
    return (
        <span className={cn(TEXT_BADGE, "text-muted-foreground")}>
            {compactNum(assignment.yield_lmd_per_day)} LMD
            {assignment.yield_exp_per_day > 0 && <> · {compactNum(assignment.yield_exp_per_day)} EXP</>}
            <PerDay className="opacity-60" />
        </span>
    );
}

export function BasePanel({ improvements, accent }: IProps) {
    const { base } = improvements;
    if (!base.optimal && !base.rotation) {
        return (
            <div className={PANEL_PADDING}>
                <EmptyHint>No base layout synced yet. Hit refresh once you're in your base.</EmptyHint>
            </div>
        );
    }

    // Difference between the player's live base and the optimal peak.
    const curVal = base.current?.yield_total_value ?? 0;
    const optVal = base.optimal?.yield_total_value ?? 0;
    const gain = optVal - curVal;

    return (
        <div className={`${PANEL_PADDING} flex flex-col gap-5`}>
            {base.layout.length > 0 && (
                <div className="flex flex-col gap-2">
                    <SectionHeader title="Current layout" accent={accent} />
                    <div className="flex flex-wrap gap-1.5">
                        {base.layout.map((l) => (
                            <span key={l.room_type} className={cn("flex items-center gap-1.5 rounded-md border border-border/40 bg-muted/15 px-2 py-1", TEXT_BODY)}>
                                <span className="font-medium">{roomLabel(l.room_type)}</span>
                                <span className={cn(TEXT_BADGE, "text-muted-foreground")}>×{l.count}</span>
                                <span className={cn(TEXT_BADGE, "text-muted-foreground")}>L{l.levels.join(", L")}</span>
                            </span>
                        ))}
                    </div>
                </div>
            )}

            {/* Current vs optimal headline + comparison entry point */}
            {base.current && base.optimal && (
                <div className="flex flex-col gap-2 rounded-md border border-border/40 bg-muted/10 p-3">
                    <div className="flex flex-wrap items-center justify-between gap-2">
                        <span className={cn(TEXT_KICKER)} style={{ color: `color-mix(in oklch, ${accent} 60%, var(--foreground))` }}>
                            Current vs optimal
                        </span>
                        {gain > 0 && <Pill color={accent}>+{compactNum(gain)} value/day</Pill>}
                    </div>
                    <div className="flex flex-wrap items-baseline gap-x-2 gap-y-1">
                        <YieldSummary assignment={base.current} />
                        <span className="text-muted-foreground/50">→</span>
                        <span className="font-mono font-semibold text-sm tabular-nums" style={{ color: `color-mix(in oklch, ${accent} 70%, var(--foreground))` }}>
                            {compactNum(base.optimal.yield_lmd_per_day)} LMD
                            {base.optimal.yield_exp_per_day > 0 && <> · {compactNum(base.optimal.yield_exp_per_day)} EXP</>}
                            <PerDay className="font-sans text-muted-foreground/60" />
                        </span>
                        <span className={cn(TEXT_BADGE, "text-muted-foreground")}>({signedCompact(base.optimal.total_production_efficiency - base.current.total_production_efficiency)}% efficiency)</span>
                    </div>

                    {/* The entire base plan in a dialog - available on every screen
                        size, including desktop, for a focused full-screen view. */}
                    <Dialog>
                        <DialogTrigger className={cn("self-start rounded-md border border-border/45 bg-background/60 px-2 py-1 transition-colors hover:border-foreground/25", TEXT_KICKER, "text-muted-foreground hover:text-foreground")}>View full base plan →</DialogTrigger>
                        <DialogPopup className="max-w-3xl">
                            <DialogTitle className="px-6 pt-6 text-base">Base optimization plan</DialogTitle>
                            <DialogPanel>
                                <BasePlanBody base={base} accent={accent} />
                            </DialogPanel>
                        </DialogPopup>
                    </Dialog>

                    {/* Inline room-by-room comparison on wide screens. */}
                    <div className="mt-1 hidden border-border/30 border-t pt-3 2xl:block">
                        <BaseComparison current={base.current} optimal={base.optimal} accent={accent} />
                    </div>
                </div>
            )}

            <RotationSection rotation={base.rotation} accent={accent} />
            <PeakSection optimal={base.optimal} accent={accent} />
        </div>
    );
}

/** The Control Center / room-by-room comparison + rotation plan + peak staffing -
 *  the full base optimization, reused inline and in the full-plan dialog. */
function BasePlanBody({ base, accent }: { base: IBaseImprovements; accent: string }) {
    return (
        <div className="flex flex-col gap-5">
            {base.current && base.optimal && <BaseComparison current={base.current} optimal={base.optimal} accent={accent} />}
            <RotationSection rotation={base.rotation} accent={accent} />
            <PeakSection optimal={base.optimal} accent={accent} />
        </div>
    );
}

function RotationSection({ rotation, accent }: { rotation: IRotation | null; accent: string }) {
    if (!rotation || rotation.rooms.length === 0) return null;
    return (
        <div className="flex flex-col gap-3">
            <SectionHeader title="Sustained 24/7 rotation" count={`+${rotation.sustained_efficiency.toFixed(1)}% sustained`} accent={accent} />
            <p className={cn(TEXT_META, "text-muted-foreground")}>
                Staggered rotation: keep the main team working, and when you log in swap only the <span className="font-medium">⚡ first operator</span> in each room (the one whose morale runs low soonest) for its backup - never a whole team at once. Times below are roughly how long each operator works before it needs
                rest.
            </p>
            <div className="flex flex-col gap-2">
                {rotation.rooms.map((room) => (
                    <RotationRoomRow key={room.slot_id} room={room} accent={accent} />
                ))}
            </div>
            <RotationSetsSection sets={rotation.sets} accent={accent} />
        </div>
    );
}

/** The rotation as a few overlapping staffings to cycle through, so you never swap
 * the whole base at once. Each set rests one operator per room (covered by a backup)
 * and consecutive sets share all but one operator. */
function RotationSetsSection({ sets, accent }: { sets: IRotationSet[] | undefined; accent: string }) {
    if (!sets || sets.length < 2) return null;
    return (
        <div className="mt-1 flex flex-col gap-2 border-border/30 border-t pt-3">
            <span className={cn(TEXT_KICKER, "text-muted-foreground")}>Overlapping sets to cycle through</span>
            <p className={cn(TEXT_META, "text-muted-foreground")}>
                Don't break the whole base at once. Cycle through these {sets.length} sets one swap at a time - each set rests only the operators that actually run low on morale (covered by a backup), while low-drain operators keep working. Consecutive sets share all but one operator per room.
            </p>
            <div className="flex flex-col gap-2">
                {sets.map((set, i) => (
                    <RotationSetRow key={set.rooms.map((r) => r.resting?.operator_id ?? r.slot_id).join("-")} set={set} index={i} accent={accent} />
                ))}
            </div>
        </div>
    );
}

function RotationSetRow({ set, index, accent }: { set: IRotationSet; index: number; accent: string }) {
    const resting = set.rooms.map((r) => r.resting?.name).filter((n): n is string => Boolean(n));
    return (
        <div className="flex flex-col gap-1.5 rounded-md border border-border/35 bg-muted/15 px-2.5 py-2">
            <div className="flex items-center justify-between gap-2">
                <span className={cn(TEXT_KICKER, "font-semibold")} style={{ color: accent }}>
                    Set {index + 1}
                </span>
                {resting.length > 0 && <span className={cn(TEXT_BADGE, "truncate text-muted-foreground/60")}>resting: {resting.join(", ")}</span>}
            </div>
            <div className="flex flex-col gap-1">
                {set.rooms.map((r) => (
                    <div key={r.slot_id} className="flex items-start gap-2">
                        <span className={cn("min-w-24 shrink-0 pt-0.5", TEXT_BADGE, "text-muted-foreground/65")}>{roomLabel(r.room_type)}</span>
                        <div className="flex flex-1 flex-wrap items-center gap-1">
                            {r.working.map((op) => (
                                <SetOpChip key={op.operator_id} op={op} />
                            ))}
                        </div>
                    </div>
                ))}
            </div>
        </div>
    );
}

function SetOpChip({ op }: { op: IAssignedOperator }) {
    return (
        <span className="flex items-center gap-1 rounded-md border border-border/40 bg-background/55 px-1 py-0.5">
            <span className="relative size-4 overflow-hidden rounded-sm">
                <OperatorAvatar charId={op.operator_id} name={op.name} />
            </span>
            <span className={cn(TEXT_BADGE, "max-w-[10ch] truncate font-medium")}>{op.name}</span>
        </span>
    );
}

function PeakSection({ optimal, accent }: { optimal: IBaseAssignment | null; accent: string }) {
    if (!optimal) return null;
    return (
        <div className="flex flex-col gap-3">
            <SectionHeader title="Peak single-shift" count={`+${optimal.total_production_efficiency.toFixed(1)}%`} accent={accent} />
            <p className={cn(TEXT_META, "text-muted-foreground")}>Best possible arrangement at one snapshot in time - useful for set-and-forget AFK runs.</p>
            <div className="flex items-center justify-between rounded-md border border-border/35 bg-muted/10 px-2.5 py-1.5">
                <span className={cn(TEXT_KICKER, "text-muted-foreground")}>Realized yield</span>
                <YieldSummary assignment={optimal} />
            </div>
            <div className="flex flex-col gap-2">
                {optimal.rooms.map((room) => (
                    <RoomRow key={`${room.slot_id}-${room.room_type}`} room={room} accent={accent} />
                ))}
            </div>
        </div>
    );
}

/** One production room's rotation plan: who to swap first, when, and the backup. */
function RotationRoomRow({ room, accent }: { room: IRoomRotation; accent: string }) {
    return (
        <div className="flex items-center gap-2 rounded-md border border-border/35 bg-muted/15 px-2.5 py-2">
            <span className="min-w-24 shrink-0">
                <span className={cn(TEXT_BODY, "font-semibold")}>{roomLabel(room.room_type)}</span>
                <span className={cn("ml-1.5", TEXT_BADGE, "text-muted-foreground/55")}>{room.slot_id}</span>
            </span>
            <div className="flex flex-1 flex-wrap items-center gap-1.5">
                {room.members.map((m, i) => (
                    <Tooltip key={m.operator.operator_id}>
                        <TooltipTrigger
                            render={
                                <span className={cn("flex items-center gap-1.5 rounded-md border bg-background/65 px-1.5 py-0.5", i === 0 ? "border-amber-500/55" : "border-border/40")}>
                                    {i === 0 && <span className="text-amber-500/90 text-xs">⚡</span>}
                                    <span className="relative size-5 overflow-hidden rounded-sm">
                                        <OperatorAvatar charId={m.operator.operator_id} name={m.operator.name} />
                                    </span>
                                    <span className={cn(TEXT_BODY, "max-w-[10ch] truncate font-medium")}>{m.operator.name}</span>
                                    <span className={cn(TEXT_BADGE, "font-mono text-muted-foreground/70 tabular-nums")}>~{m.lasts_hours.toFixed(0)}h</span>
                                </span>
                            }
                        />
                        <TooltipContent sideOffset={4}>
                            <p>
                                {i === 0 ? "Swap this one first - " : ""}
                                {m.operator.name} works ~{m.lasts_hours.toFixed(0)}h before its morale runs low.
                            </p>
                        </TooltipContent>
                    </Tooltip>
                ))}
                {room.backup && (
                    <>
                        <span className="px-0.5 text-muted-foreground/50" style={{ color: `color-mix(in oklch, ${accent} 55%, var(--muted-foreground))` }}>
                            ⇄
                        </span>
                        <Tooltip>
                            <TooltipTrigger
                                render={
                                    <span className="flex items-center gap-1.5 rounded-md border border-border/45 border-dashed bg-background/40 px-1.5 py-0.5">
                                        <span className="relative size-5 overflow-hidden rounded-sm opacity-80">
                                            <OperatorAvatar charId={room.backup.operator_id} name={room.backup.name} />
                                        </span>
                                        <span className={cn(TEXT_BODY, "max-w-[10ch] truncate font-medium text-muted-foreground")}>{room.backup.name}</span>
                                        <span className={cn(TEXT_BADGE, "text-muted-foreground/55")}>backup</span>
                                    </span>
                                }
                            />
                            <TooltipContent sideOffset={4}>
                                <p>Rotate {room.backup.name} in when the ⚡ first operator needs rest.</p>
                            </TooltipContent>
                        </Tooltip>
                    </>
                )}
            </div>
        </div>
    );
}

function RoomRow({ room, accent, compact }: { room: IRoomAssignment; accent: string; compact?: boolean }) {
    const label = roomLabel(room.room_type);
    const yieldLabel = roomYieldLabel(room);
    return (
        <div className={cn("flex items-center gap-2 rounded-md border border-border/35 bg-muted/15", compact ? "px-2 py-1.5" : "px-2.5 py-2")}>
            <div className="flex min-w-28 flex-col">
                <div className="flex items-center gap-1.5">
                    <span className={cn(TEXT_BODY, "font-semibold")}>{label}</span>
                    <span className={cn("rounded-sm border border-border/40 bg-background px-1 py-0.5 text-muted-foreground", TEXT_BADGE)}>L{room.level}</span>
                    {room.locked && room.operators.length > 0 && (
                        <Tooltip>
                            <TooltipTrigger render={<span className={cn("rounded-sm border border-amber-500/40 bg-amber-500/10 px-1 py-0.5 text-amber-500/90", TEXT_BADGE)}>🔒 fixed</span>} />
                            <TooltipContent sideOffset={4}>
                                <p>Fixed synergy squad - these operators depend on each other (e.g. Shamare + Tequila, or Texas + Lappland) and can't be swapped without breaking the combo.</p>
                            </TooltipContent>
                        </Tooltip>
                    )}
                </div>
                <div className={cn("flex items-center gap-1.5 text-muted-foreground", TEXT_BADGE)}>
                    <span className="opacity-65">{room.slot_id}</span>
                    {room.formula_type && (
                        <Tooltip>
                            <TooltipTrigger render={<span className="opacity-80">{room.formula_type}</span>} />
                            <TooltipContent sideOffset={4}>
                                <p>Production formula assigned to this room.</p>
                            </TooltipContent>
                        </Tooltip>
                    )}
                </div>
            </div>
            <div className="flex flex-1 flex-wrap gap-1">
                {room.operators.length === 0 && <span className={cn(TEXT_KICKER, "text-muted-foreground/60")}>No assignment</span>}
                {room.operators.map((op) => (
                    <Tooltip key={op.operator_id}>
                        <TooltipTrigger
                            render={
                                <span className="flex items-center gap-1.5 rounded-md border border-border/40 bg-background/65 px-1.5 py-0.5">
                                    <span className="relative size-5 overflow-hidden rounded-sm">
                                        <OperatorAvatar charId={op.operator_id} name={op.name} />
                                    </span>
                                    <span className={cn(TEXT_BODY, "max-w-[11ch] truncate font-medium")}>{op.name}</span>
                                </span>
                            }
                        />
                        <TooltipContent sideOffset={4}>
                            <p>{op.name}</p>
                        </TooltipContent>
                    </Tooltip>
                ))}
            </div>
            <div className="flex shrink-0 flex-col items-end gap-0.5">
                <span className={cn("font-semibold", TEXT_BODY, "font-mono tabular-nums")} style={{ color: `color-mix(in oklch, ${accent} 65%, var(--foreground))` }}>
                    +{room.total_efficiency.toFixed(1)}%
                </span>
                {room.room_type === "CONTROL" && (
                    <Tooltip>
                        <TooltipTrigger render={<span className={cn(TEXT_BADGE, "text-muted-foreground/70")}>global buff</span>} />
                        <TooltipContent sideOffset={4}>
                            <p>Production boost the Control Center applies to every Factory & Trading Post in the base.</p>
                        </TooltipContent>
                    </Tooltip>
                )}
                {room.order_value > 0 && (
                    <Tooltip>
                        <TooltipTrigger render={<span className={cn(TEXT_BADGE, "text-amber-500/85")}>+{room.order_value.toFixed(0)}% value</span>} />
                        <TooltipContent sideOffset={4}>
                            <p>Order value (LMD per order, e.g. Proviso) - raises LMD without adding order speed.</p>
                        </TooltipContent>
                    </Tooltip>
                )}
                {yieldLabel && (
                    <Tooltip>
                        <TooltipTrigger render={<span className={cn(TEXT_BADGE, "cursor-help text-muted-foreground/70")}>{yieldLabel}</span>} />
                        <TooltipContent sideOffset={4}>
                            <p className="max-w-xs">Averaged over a ~36h staggered-rotation cycle (sustained daily average, not a constant peak).{yieldLabel.includes("*") ? " * Trading-post LMD assumes it's gold-supplied." : ""}</p>
                        </TooltipContent>
                    </Tooltip>
                )}
            </div>
        </div>
    );
}
