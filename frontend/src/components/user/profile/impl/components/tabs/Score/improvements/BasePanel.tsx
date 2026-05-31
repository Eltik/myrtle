import { Dialog, DialogPanel, DialogPopup, DialogTitle, DialogTrigger } from "#/components/ui/dialog";
import { OperatorAvatar } from "#/components/ui/operator-avatar";
import { Tooltip, TooltipContent, TooltipTrigger } from "#/components/ui/tooltip";
import type { IBaseAssignment, IImprovementsResponse, IRoomAssignment } from "#/lib/api/user";
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

/** "X LMD/day + Y EXP/day" summary for an assignment's realized yield. */
function YieldSummary({ assignment }: { assignment: IBaseAssignment }) {
    return (
        <span className={cn(TEXT_BADGE, "text-muted-foreground")}>
            {compactNum(assignment.yield_lmd_per_day)} LMD
            {assignment.yield_exp_per_day > 0 && <> · {compactNum(assignment.yield_exp_per_day)} EXP</>}
            <span className="opacity-60">/day</span>
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
                            <span className="font-sans text-muted-foreground/60">/day</span>
                        </span>
                        <span className={cn(TEXT_BADGE, "text-muted-foreground")}>({signedCompact(base.optimal.total_production_efficiency - base.current.total_production_efficiency)}% efficiency)</span>
                    </div>

                    {/* Inline comparison when there's room; dialog otherwise. */}
                    <div className="hidden 2xl:block">
                        <div className="mt-1 border-border/30 border-t pt-3">
                            <BaseComparison current={base.current} currentShiftA={base.current_rotation?.shift_a ?? null} currentShiftB={base.current_rotation?.shift_b ?? null} optimal={base.optimal} shiftA={base.rotation?.shift_a ?? null} shiftB={base.rotation?.shift_b ?? null} accent={accent} />
                        </div>
                    </div>
                    <div className="2xl:hidden">
                        <Dialog>
                            <DialogTrigger className={cn("self-start rounded-md border border-border/45 bg-background/60 px-2 py-1 transition-colors hover:border-foreground/25", TEXT_KICKER, "text-muted-foreground hover:text-foreground")}>Compare room by room →</DialogTrigger>
                            <DialogPopup className="max-w-2xl">
                                <DialogTitle className="px-6 pt-6 text-base">Current base vs optimized</DialogTitle>
                                <DialogPanel>
                                    <BaseComparison current={base.current} currentShiftA={base.current_rotation?.shift_a ?? null} currentShiftB={base.current_rotation?.shift_b ?? null} optimal={base.optimal} shiftA={base.rotation?.shift_a ?? null} shiftB={base.rotation?.shift_b ?? null} accent={accent} />
                                </DialogPanel>
                            </DialogPopup>
                        </Dialog>
                    </div>
                </div>
            )}

            {base.rotation && (
                <div className="flex flex-col gap-3">
                    <SectionHeader title="Optimal 2-shift rotation" count={`+${base.rotation.sustained_efficiency.toFixed(1)}% sustained`} accent={accent} />
                    <p className={cn(TEXT_META, "text-muted-foreground")}>Average of shift A and shift B. This is what your base score is computed against - run operators on these rotations to hit it.</p>
                    <div className="grid gap-3 lg:grid-cols-2">
                        <ShiftBlock title="Shift A" assignment={base.rotation.shift_a} accent={accent} />
                        <ShiftBlock title="Shift B" assignment={base.rotation.shift_b} accent={accent} />
                    </div>
                </div>
            )}

            {base.optimal && (
                <div className="flex flex-col gap-3">
                    <SectionHeader title="Peak single-shift" count={`+${base.optimal.total_production_efficiency.toFixed(1)}%`} accent={accent} />
                    <p className={cn(TEXT_META, "text-muted-foreground")}>Best possible arrangement at one snapshot in time - useful for set-and-forget AFK runs.</p>
                    <div className="flex items-center justify-between rounded-md border border-border/35 bg-muted/10 px-2.5 py-1.5">
                        <span className={cn(TEXT_KICKER, "text-muted-foreground")}>Realized yield</span>
                        <YieldSummary assignment={base.optimal} />
                    </div>
                    <div className="flex flex-col gap-2">
                        {base.optimal.rooms.map((room) => (
                            <RoomRow key={`${room.slot_id}-${room.room_type}`} room={room} accent={accent} />
                        ))}
                    </div>
                </div>
            )}
        </div>
    );
}

function ShiftBlock({ title, assignment, accent }: { title: string; assignment: IBaseAssignment; accent: string }) {
    return (
        <div className="flex flex-col gap-2 rounded-md border border-border/40 bg-muted/10 p-3">
            <div className="flex items-center justify-between">
                <span className={cn(TEXT_KICKER)} style={{ color: `color-mix(in oklch, ${accent} 60%, var(--foreground))` }}>
                    {title}
                </span>
                <Pill color={accent}>+{assignment.total_production_efficiency.toFixed(1)}%</Pill>
            </div>
            <div className="flex items-center justify-between border-border/25 border-b pb-1.5">
                <span className={cn(TEXT_BADGE, "text-muted-foreground/70")}>Yield</span>
                <YieldSummary assignment={assignment} />
            </div>
            <div className="flex flex-col gap-1.5">
                {assignment.rooms.map((room) => (
                    <RoomRow key={`${room.slot_id}-${room.room_type}`} room={room} accent={accent} compact />
                ))}
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
                {room.order_value > 0 && (
                    <Tooltip>
                        <TooltipTrigger render={<span className={cn(TEXT_BADGE, "text-amber-500/85")}>+{room.order_value.toFixed(0)}% value</span>} />
                        <TooltipContent sideOffset={4}>
                            <p>Order value (LMD per order, e.g. Proviso) — raises LMD without adding order speed.</p>
                        </TooltipContent>
                    </Tooltip>
                )}
                {yieldLabel && <span className={cn(TEXT_BADGE, "text-muted-foreground/70")}>{yieldLabel}</span>}
            </div>
        </div>
    );
}
