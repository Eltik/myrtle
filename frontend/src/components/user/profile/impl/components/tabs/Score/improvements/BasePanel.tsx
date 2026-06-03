import { ChevronDown, Download } from "lucide-react";
import { type ReactNode, type RefObject, useRef, useState } from "react";
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "#/components/ui/collapsible";
import { Dialog, DialogPanel, DialogPopup, DialogTitle, DialogTrigger } from "#/components/ui/dialog";
import { OperatorAvatar } from "#/components/ui/operator-avatar";
import { toastManager } from "#/components/ui/toast";
import { Tooltip, TooltipContent, TooltipTrigger } from "#/components/ui/tooltip";
import type { IAssignedOperator, IBaseAssignment, IBaseImprovements, IImprovementsResponse, IRoomAssignment, IRoomRotation, IRotation, IRotationSet, IShift, IShiftRoom, IShiftRotation } from "#/lib/api/user";
import { cn } from "#/lib/utils";
import { BaseComparison } from "./BaseComparison";
import { compactNum, roomYieldLabel, signedCompact } from "./baseYield";
import { ExportPlanContent } from "./ExportPlanContent";
import { exportPlanAsImage } from "./exportPlan";
import { roomAccent, roomFormulaLabel, roomLabel } from "./roomColors";
import { EmptyHint, PANEL_PADDING, Pill, SectionHeader, TEXT_BADGE, TEXT_BODY, TEXT_KICKER, TEXT_META } from "./shared";

interface IProps {
    improvements: IImprovementsResponse;
    accent: string;
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
                        {base.layout.map((l) => {
                            const a = roomAccent(l.room_type);
                            return (
                                <span key={l.room_type} className={cn("flex items-center gap-1.5 rounded-md border px-2 py-1", TEXT_BODY)} style={{ borderColor: a.border, background: a.tint }}>
                                    <span className="size-2 rounded-[3px]" style={{ background: a.color }} />
                                    <span className="font-medium" style={{ color: a.text }}>
                                        {roomLabel(l.room_type)}
                                    </span>
                                    <span className={cn(TEXT_BADGE, "text-muted-foreground")}>×{l.count}</span>
                                    <span className={cn(TEXT_BADGE, "text-muted-foreground")}>L{l.levels.join(", L")}</span>
                                </span>
                            );
                        })}
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
                    <FullPlanDialog base={base} accent={accent} />
                </div>
            )}
        </div>
    );
}

/** The full-plan dialog plus its "Export plan" action. The exported image captures a
 *  dedicated wide poster (`ExportPlanContent`) kept off-screen, NOT the tall dialog
 *  body, so the PNG is well-proportioned rather than a long vertical ribbon. */
function FullPlanDialog({ base, accent }: { base: IBaseImprovements; accent: string }) {
    const captureRef = useRef<HTMLDivElement>(null);
    return (
        <Dialog>
            <DialogTrigger className={cn("self-start rounded-md border border-border/45 bg-background/60 px-2 py-1 transition-colors hover:border-foreground/25", TEXT_KICKER, "text-muted-foreground hover:text-foreground")}>View full base plan →</DialogTrigger>
            <DialogPopup className="max-w-3xl">
                <div className="flex items-center justify-between gap-3 px-6 pt-6">
                    <DialogTitle className="text-base">Base optimization plan</DialogTitle>
                    <ExportPlanButton targetRef={captureRef} />
                </div>
                <DialogPanel>
                    <BasePlanBody base={base} accent={accent} />
                </DialogPanel>
            </DialogPopup>
            {/* Off-screen export layout - present in the DOM so its ref is ready, but
                visually hidden and laid out at its own fixed width. */}
            <div aria-hidden className="pointer-events-none fixed top-0 left-[-99999px] opacity-0">
                <div ref={captureRef}>
                    <ExportPlanContent base={base} />
                </div>
            </div>
        </Dialog>
    );
}

/** Exports a dedicated, wide poster layout (rendered off-screen) to a PNG - capturing
 *  that instead of the tall dialog body keeps the image well-proportioned. */
function ExportPlanButton({ targetRef }: { targetRef: RefObject<HTMLDivElement | null> }) {
    const [busy, setBusy] = useState(false);
    const handleExport = async () => {
        if (busy || !targetRef.current) return;
        setBusy(true);
        try {
            await exportPlanAsImage(targetRef.current, "base-plan.png");
        } catch {
            toastManager.add({
                id: `base-export-${Date.now()}`,
                title: "Couldn't export plan",
                description: "Something went wrong rendering the image. Try again in a moment.",
                type: "error",
            });
        } finally {
            setBusy(false);
        }
    };
    return (
        <button type="button" onClick={handleExport} disabled={busy} className={cn("flex shrink-0 items-center gap-1.5 rounded-md border border-border/45 bg-background/60 px-2 py-1 transition-colors hover:border-foreground/25 disabled:opacity-60", TEXT_KICKER, "text-muted-foreground hover:text-foreground")}>
            <Download className="size-3.5" />
            {busy ? "Exporting…" : "Export plan"}
        </button>
    );
}

/**
 * The consolidated full base plan. Leads with the shift rotation (the primary,
 * actionable view) and the current-vs-optimal comparison; the sustained 24/7
 * rotation and peak single-shift are tucked into collapsible sections so the
 * dialog opens focused instead of as one long wall.
 */
function BasePlanBody({ base, accent }: { base: IBaseImprovements; accent: string }) {
    const hasShifts = Boolean(base.shift_rotation && base.shift_rotation.shifts.length > 0);
    return (
        <div className="flex flex-col gap-5">
            <ShiftRotationSection rotation={base.shift_rotation} accent={accent} />

            {base.current && base.optimal && (
                <section className="flex flex-col gap-3">
                    <SectionHeader title="Current vs optimal" accent={accent} />
                    <BaseComparison current={base.current} optimal={base.optimal} accent={accent} />
                </section>
            )}

            {base.rotation && base.rotation.rooms.length > 0 && (
                <CollapsibleSection title="Sustained 24/7 rotation" count={`+${base.rotation.sustained_efficiency.toFixed(1)}% sustained`} accent={accent} defaultOpen={!hasShifts}>
                    <RotationBody rotation={base.rotation} accent={accent} />
                </CollapsibleSection>
            )}

            {base.optimal && (
                <CollapsibleSection title="Peak single-shift" count={`+${base.optimal.total_production_efficiency.toFixed(1)}%`} accent={accent}>
                    <PeakBody optimal={base.optimal} />
                </CollapsibleSection>
            )}
        </div>
    );
}

// ─── Shared room card ─────────────────────────────────────────────────────────

/** A single room's crew framed in a shape border tinted with the building's
 *  in-game colour. */
function RoomShell({ roomType, formula, badge, right, dim, children }: { roomType: string; formula?: string | null; badge?: ReactNode; right?: ReactNode; dim?: boolean; children: ReactNode }) {
    const a = roomAccent(roomType);
    return (
        <div className={cn("flex flex-col gap-1.5 rounded-lg border-2 px-2.5 py-2", dim && "opacity-60")} style={{ borderColor: a.border, background: a.tint }}>
            <div className="flex items-center justify-between gap-2">
                <div className="flex items-center gap-1.5">
                    <span className="size-2 shrink-0 rounded-[3px]" style={{ background: a.color }} />
                    <span className={cn(TEXT_KICKER)} style={{ color: a.text }}>
                        {roomFormulaLabel(roomType, formula)}
                    </span>
                    {badge}
                </div>
                {right}
            </div>
            {children}
        </div>
    );
}

/** Legend mapping each room colour to its name, shown above the rotation. */
function RoomColorLegend({ types }: { types: string[] }) {
    const seen = [...new Set(types)];
    if (seen.length === 0) return null;
    return (
        <div className="flex flex-wrap items-center gap-x-3 gap-y-1 rounded-md border border-border/30 bg-muted/10 px-2.5 py-1.5">
            {seen.map((t) => {
                const a = roomAccent(t);
                return (
                    <span key={t} className="flex items-center gap-1.5">
                        <span className="size-2.5 rounded-[3px] border" style={{ background: a.tint, borderColor: a.border }} />
                        <span className={cn(TEXT_BADGE)} style={{ color: a.text }}>
                            {roomLabel(t)}
                        </span>
                    </span>
                );
            })}
        </div>
    );
}

/** A collapsible secondary section (sustained rotation, peak) - keeps the dialog
 *  focused on the rotation while leaving the detail one click away. */
function CollapsibleSection({ title, count, accent, defaultOpen = false, children }: { title: string; count?: ReactNode; accent: string; defaultOpen?: boolean; children: ReactNode }) {
    const [open, setOpen] = useState(defaultOpen);
    return (
        <Collapsible open={open} onOpenChange={setOpen} className="flex flex-col gap-3">
            <CollapsibleTrigger className="flex items-center justify-between gap-2 border-border/40 border-b pb-1.5 text-left transition-colors hover:border-border">
                <span className={cn(TEXT_KICKER)} style={{ color: `color-mix(in oklch, ${accent} 60%, var(--foreground))` }}>
                    {title}
                </span>
                <span className="flex items-center gap-2">
                    {count !== undefined && <span className={cn(TEXT_BADGE, "rounded-md border border-border/40 bg-muted/30 px-1.5 py-0.5 text-muted-foreground")}>{count}</span>}
                    <ChevronDown className={cn("size-3.5 text-muted-foreground transition-transform", open && "rotate-180")} />
                </span>
            </CollapsibleTrigger>
            <CollapsibleContent>
                <div className="flex flex-col gap-3 pt-0.5">{children}</div>
            </CollapsibleContent>
        </Collapsible>
    );
}

// ─── Shift rotation (primary) ─────────────────────────────────────────────────

/** A chip whose border/tint signals whether the operator should be added (green),
 *  removed (red, struck through), or kept (neutral). */
function ShiftOpChip({ op, tone }: { op: IAssignedOperator; tone: "add" | "remove" | "keep" }) {
    const toneCls = tone === "add" ? "border-emerald-500/55 bg-emerald-500/15" : tone === "remove" ? "border-rose-500/45 bg-rose-500/15" : "border-border/40 bg-background/55";
    return (
        <span className={cn("flex items-center gap-1 rounded-md border px-1 py-0.5", toneCls)}>
            <span className="relative size-4 overflow-hidden rounded-sm">
                <OperatorAvatar charId={op.operator_id} name={op.name} />
            </span>
            <span className={cn(TEXT_BADGE, "max-w-[10ch] truncate font-medium", tone === "remove" && "text-rose-500/80 line-through")}>{op.name}</span>
        </span>
    );
}

/** Recommended 3-shift rotation vs the player's saved in-game presets. */
function ShiftRotationSection({ rotation, accent }: { rotation: IShiftRotation | null | undefined; accent: string }) {
    if (!rotation || rotation.shifts.length === 0) return null;
    const types = rotation.shifts.flatMap((s) => s.rooms.map((r) => r.room_type));
    return (
        <div className="flex flex-col gap-3">
            <SectionHeader title="Shift rotation vs your presets" accent={accent} />
            <p className={cn(TEXT_META, "text-muted-foreground")}>
                Run three teams per room so each rests one shift. Each card is one room, outlined in its in-game colour. <span className="font-medium text-emerald-500/85">Green</span> = add to your preset; <span className="text-rose-500/85 line-through">red</span> = remove. Power plants swap every shift; the Control
                Center runs two shifts on, one off.
            </p>
            <RoomColorLegend types={types} />
            <div className="flex flex-col gap-3">
                {rotation.shifts.map((shift) => (
                    <ShiftCard key={shift.index} shift={shift} />
                ))}
            </div>
        </div>
    );
}

function ShiftCard({ shift }: { shift: IShift }) {
    return (
        <div className="flex flex-col gap-2">
            <div className="flex items-center gap-2">
                <span className={cn(TEXT_KICKER, "shrink-0 rounded bg-muted/40 px-1.5 py-0.5 text-foreground/75")}>Shift {shift.index}</span>
                <span className="h-px flex-1 bg-border/40" />
            </div>
            <div className="grid grid-cols-1 gap-2 sm:grid-cols-2">
                {shift.rooms.map((room) => (
                    <ShiftRoomCard key={room.slot_id} room={room} />
                ))}
            </div>
        </div>
    );
}

function ShiftRoomCard({ room }: { room: IShiftRoom }) {
    const swapOutIds = new Set(room.swap_out.map((o) => o.operator_id));
    const swapInIds = new Set(room.swap_in.map((o) => o.operator_id));
    const hasPreset = room.current.length > 0;
    const badge = !room.active ? <span className={cn(TEXT_BADGE, "text-amber-500/80")}>off this shift</span> : hasPreset && room.matches ? <span className="text-[10px] text-emerald-500/80">✓ matches yours</span> : undefined;
    return (
        <RoomShell roomType={room.room_type} formula={room.formula_type} dim={!room.active} badge={badge}>
            <div className="flex flex-wrap items-center gap-1">
                {room.recommended.length === 0 && <span className={cn(TEXT_BADGE, "text-muted-foreground/50")}>-</span>}
                {room.recommended.map((op) => (
                    <ShiftOpChip key={op.operator_id} op={op} tone={swapInIds.has(op.operator_id) ? "add" : "keep"} />
                ))}
            </div>
            {hasPreset && !room.matches && (
                <div className="flex flex-wrap items-center gap-1 border-border/25 border-t pt-1.5">
                    <span className={cn(TEXT_BADGE, "min-w-9 text-muted-foreground/55")}>yours</span>
                    {room.current.map((op) => (
                        <ShiftOpChip key={op.operator_id} op={op} tone={swapOutIds.has(op.operator_id) ? "remove" : "keep"} />
                    ))}
                </div>
            )}
            {!hasPreset && room.active && <span className={cn(TEXT_BADGE, "text-muted-foreground/45")}>no saved preset for this shift</span>}
        </RoomShell>
    );
}

// ─── Sustained 24/7 rotation (secondary) ──────────────────────────────────────

function RotationBody({ rotation, accent }: { rotation: IRotation; accent: string }) {
    return (
        <>
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
        </>
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
                {set.rooms.map((r) => {
                    const a = roomAccent(r.room_type);
                    return (
                        <div key={r.slot_id} className="flex items-start gap-2">
                            <span className={cn("min-w-24 shrink-0 pt-0.5", TEXT_BADGE)} style={{ color: a.text }}>
                                {roomLabel(r.room_type)}
                            </span>
                            <div className="flex flex-1 flex-wrap items-center gap-1">
                                {r.working.map((op) => (
                                    <ShiftOpChip key={op.operator_id} op={op} tone="keep" />
                                ))}
                            </div>
                        </div>
                    );
                })}
            </div>
        </div>
    );
}

/** One production room's rotation plan: who to swap first, when, and the backup. */
function RotationRoomRow({ room, accent }: { room: IRoomRotation; accent: string }) {
    const a = roomAccent(room.room_type);
    return (
        <div className="flex items-center gap-2 rounded-lg border-2 px-2.5 py-2" style={{ borderColor: a.border, background: a.tint }}>
            <span className="flex min-w-24 shrink-0 items-center gap-1.5">
                <span className="size-2 shrink-0 rounded-[3px]" style={{ background: a.color }} />
                <span className={cn(TEXT_BODY, "font-semibold")} style={{ color: a.text }}>
                    {roomLabel(room.room_type)}
                </span>
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
                        <span className="px-0.5" style={{ color: `color-mix(in oklch, ${accent} 55%, var(--muted-foreground))` }}>
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

// ─── Peak single-shift (secondary) ────────────────────────────────────────────

function PeakBody({ optimal }: { optimal: IBaseAssignment }) {
    return (
        <>
            <p className={cn(TEXT_META, "text-muted-foreground")}>Best possible arrangement at one snapshot in time - useful for set-and-forget AFK runs.</p>
            <div className="flex items-center justify-between rounded-md border border-border/35 bg-muted/10 px-2.5 py-1.5">
                <span className={cn(TEXT_KICKER, "text-muted-foreground")}>Realized yield</span>
                <YieldSummary assignment={optimal} />
            </div>
            <div className="flex flex-col gap-2">
                {optimal.rooms.map((room) => (
                    <RoomRow key={`${room.slot_id}-${room.room_type}`} room={room} />
                ))}
            </div>
        </>
    );
}

function RoomRow({ room }: { room: IRoomAssignment }) {
    const a = roomAccent(room.room_type);
    const yieldLabel = roomYieldLabel(room);
    return (
        <div className="flex items-center gap-2 rounded-lg border-2 px-2.5 py-2" style={{ borderColor: a.border, background: a.tint }}>
            <div className="flex min-w-28 flex-col">
                <div className="flex items-center gap-1.5">
                    <span className="size-2 shrink-0 rounded-[3px]" style={{ background: a.color }} />
                    <span className={cn(TEXT_BODY, "font-semibold")} style={{ color: a.text }}>
                        {roomLabel(room.room_type)}
                    </span>
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
                <div className={cn("flex items-center gap-1.5 pl-3.5 text-muted-foreground", TEXT_BADGE)}>
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
                <span className={cn("font-semibold", TEXT_BODY, "font-mono tabular-nums")} style={{ color: a.strong }}>
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
