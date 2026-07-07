import { ChevronDown, Download } from "lucide-react";
import { type ReactNode, type RefObject, useRef, useState } from "react";
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "#/components/ui/collapsible";
import { Dialog, DialogPanel, DialogPopup, DialogTitle, DialogTrigger } from "#/components/ui/dialog";
import { toastManager } from "#/components/ui/toast";
import { Tooltip, TooltipContent, TooltipTrigger } from "#/components/ui/tooltip";
import type { IBaseAssignment, IBaseImprovements, IImprovementsResponse, IPerceptionPlan, IRoomAssignment, IRoomRotation, IRotation, IRotationSet, IShift, IShiftRotation } from "#/lib/api/user";
import { cn } from "#/lib/utils";
import { BaseComparison } from "./BaseComparison";
import { compactNum, roomYieldLabel, signedCompact } from "./baseYield";
import { ExportPlanContent } from "./ExportPlanContent";
import { exportPlanAsImage } from "./exportPlan";
import { MiniChip, RoomBlock, ShiftRoomBlock } from "./planParts";
import { roomAccent, roomLabel } from "./roomColors";
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

/** Current → optimal daily-yield line: the realized current yield, the optimal LMD/EXP,
 *  and the efficiency delta between them. */
function YieldHeadline({ current, optimal, accent }: { current: IBaseAssignment; optimal: IBaseAssignment; accent: string }) {
    return (
        <div className="flex flex-wrap items-baseline gap-x-2 gap-y-1">
            <YieldSummary assignment={current} />
            <span className="text-muted-foreground/50">→</span>
            <span className="font-mono font-semibold text-sm tabular-nums" style={{ color: `color-mix(in oklch, ${accent} 70%, var(--foreground))` }}>
                {compactNum(optimal.yield_lmd_per_day)} LMD
                {optimal.yield_exp_per_day > 0 && <> · {compactNum(optimal.yield_exp_per_day)} EXP</>}
                <PerDay className="font-sans text-muted-foreground/60" />
            </span>
            <span className={cn(TEXT_BADGE, "text-muted-foreground")}>({signedCompact(optimal.total_production_efficiency - current.total_production_efficiency)}% efficiency)</span>
        </div>
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

            {base.current && base.optimal && (
                <div className="flex flex-col gap-2 rounded-md border border-border/40 bg-muted/10 p-3">
                    <div className="flex flex-wrap items-center justify-between gap-2">
                        <span className={cn(TEXT_KICKER)} style={{ color: `color-mix(in oklch, ${accent} 60%, var(--foreground))` }}>
                            Current vs optimal
                        </span>
                        {gain > 0 && <Pill color={accent}>+{compactNum(gain)} value/day</Pill>}
                    </div>
                    <YieldHeadline current={base.current} optimal={base.optimal} accent={accent} />

                    {/* Dialog opens the full plan as a focused full-screen view on every
                        screen size, desktop included. */}
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
            <DialogPopup className="max-w-4xl">
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
            <div aria-hidden className="pointer-events-none fixed top-0 -left-24999.75 opacity-0">
                <div ref={captureRef}>
                    <ExportPlanContent base={base} />
                </div>
            </div>
        </Dialog>
    );
}

/** Renders the off-screen poster (`targetRef`) to a downloadable PNG. */
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
 * The consolidated full base plan, laid out like the downloadable poster: a compact
 * header (layout + yield), then the 3-shift rotation as a side-by-side grid of
 * colour-accented room blocks. The sustained 24/7 rotation, peak single-shift, resource
 * economy, and full comparison are tucked into collapsible sections so the dialog opens
 * focused instead of as one long wall.
 */
function BasePlanBody({ base, accent }: { base: IBaseImprovements; accent: string }) {
    const hasShifts = Boolean(base.shift_rotation && base.shift_rotation.shifts.length > 0);
    return (
        <div className="flex flex-col gap-4">
            <PlanHeader base={base} accent={accent} />

            {hasShifts && base.shift_rotation ? <ShiftPoster rotation={base.shift_rotation} /> : base.optimal && <PeakGrid optimal={base.optimal} />}

            {base.rotation && base.rotation.rooms.length > 0 && (
                <CollapsibleSection title="Sustained 24/7 rotation" count={`+${base.rotation.sustained_efficiency.toFixed(1)}% sustained`} accent={accent} defaultOpen={!hasShifts}>
                    <RotationBody rotation={base.rotation} accent={accent} />
                </CollapsibleSection>
            )}

            {hasShifts && base.optimal && (
                <CollapsibleSection title="Peak single-shift" count={`+${base.optimal.total_production_efficiency.toFixed(1)}%`} accent={accent}>
                    <PeakGrid optimal={base.optimal} />
                </CollapsibleSection>
            )}

            {base.perception && base.perception.consumers.length > 0 && (
                <CollapsibleSection title="Resource economy (max ceiling)" count={`+${Math.max(...base.perception.consumers.map((c) => c.bonus_pct)).toFixed(0)}% peak`} accent={accent}>
                    <PerceptionSection plan={base.perception} />
                </CollapsibleSection>
            )}

            {base.current && base.optimal && (
                <CollapsibleSection title="Current vs optimal detail" accent={accent}>
                    <BaseComparison current={base.current} optimal={base.optimal} accent={accent} />
                </CollapsibleSection>
            )}
        </div>
    );
}

/** Poster-style header: the base layout as colour chips (doubling as the room-colour
 *  legend) and the current → optimal yield headline. */
function PlanHeader({ base, accent }: { base: IBaseImprovements; accent: string }) {
    return (
        <div className="flex flex-col gap-2.5 border-border/40 border-b pb-3">
            {base.layout.length > 0 && (
                <div className="flex flex-wrap items-center gap-1.5">
                    {base.layout.map((l) => {
                        const a = roomAccent(l.room_type);
                        return (
                            <span key={l.room_type} className="flex items-center gap-1 rounded border px-1.5 py-0.5 text-[10px]" style={{ borderColor: a.border, background: a.tint }}>
                                <span className="size-1.5 rounded-xs" style={{ background: a.color }} />
                                <span className="font-medium" style={{ color: a.text }}>
                                    {roomLabel(l.room_type)}
                                </span>
                                <span className="text-muted-foreground">×{l.count}</span>
                            </span>
                        );
                    })}
                </div>
            )}
            {base.current && base.optimal && <YieldHeadline current={base.current} optimal={base.optimal} accent={accent} />}
        </div>
    );
}

// ─── Shift rotation (primary) ─────────────────────────────────────────────────

/** The recommended 3-shift rotation as a side-by-side poster (mirrors the export image):
 *  one column per shift, each a stack of colour-accented room blocks. Green = add vs your
 *  preset, red = remove. */
function ShiftPoster({ rotation }: { rotation: IShiftRotation }) {
    if (rotation.shifts.length === 0) return null;
    const sustained = new Set(rotation.sustained.map((o) => o.operator_id));
    return (
        <div className="flex flex-col gap-2.5">
            <p className={cn(TEXT_META, "text-muted-foreground")}>
                Each production team (Team A/B/C per room pair) works a 24h block - two shifts in a row - then rests one; at every login you swap exactly one team per room group. Power plants, the Office, Reception and the Control Center alternate two squads (Squad 1 / Squad 2) - the swap you make each login.{" "}
                <span className="font-medium text-emerald-500/85">Green</span> = add to your preset; <span className="text-rose-500/85 line-through">red</span> = remove; ≈ yours = your team is close enough to keep (the small % shows what the recommended team would add).
                {sustained.size > 0 && (
                    <>
                        {" "}
                        Operators tagged <span className="rounded-sm bg-amber-500/20 px-0.5 font-semibold text-[9px] text-amber-500/90">24/7</span> are held at full morale every shift by a morale-swap manager (Fiammetta).
                    </>
                )}
            </p>
            <div className="grid grid-cols-1 gap-2.5 sm:grid-cols-3">
                {rotation.shifts.map((shift) => (
                    <ShiftColumn key={shift.index} shift={shift} sustained={sustained} />
                ))}
            </div>
        </div>
    );
}

function ShiftColumn({ shift, sustained }: { shift: IShift; sustained?: Set<string> }) {
    return (
        <div className="flex flex-col gap-1.5 rounded-md border border-border/35 bg-muted/10 p-2">
            <span className="font-semibold text-[11px]">Shift {shift.index}</span>
            {shift.rooms.map((room) => (
                <ShiftRoomBlock key={room.slot_id} room={room} interactive sustained={sustained} />
            ))}
        </div>
    );
}

// ─── Resource economy (Perception Information) ────────────────────────────────

/** The base-wide resource economy: support operators to station outside production, and the
 *  production operators they power. A niche, max-output ceiling - not a normal-base default. */
function PerceptionSection({ plan }: { plan: IPerceptionPlan }) {
    return (
        <>
            <p className={cn(TEXT_META, "text-muted-foreground")}>Operators resting in your dormitories fuel a shared pool that supercharges specific production operators. Station the support crew, and the powered operators climb far past a normal team - a high-ceiling, high-effort strategy most players can skip.</p>
            {plan.support.length > 0 && (
                <div className="flex flex-col gap-1.5">
                    <span className={cn(TEXT_KICKER, "text-muted-foreground")}>Station support</span>
                    <div className="flex flex-wrap gap-1">
                        {plan.support.map((s) => (
                            <MiniChip
                                key={s.operator.operator_id}
                                op={s.operator}
                                suffix={
                                    <span className="font-mono text-[9px]" style={{ color: roomAccent(s.room_type).text }}>
                                        → {roomLabel(s.room_type)}
                                    </span>
                                }
                                tip={
                                    <p>
                                        Station {s.operator.name} in the {roomLabel(s.room_type)} to feed the pool.
                                    </p>
                                }
                            />
                        ))}
                    </div>
                </div>
            )}
            <div className="flex flex-col gap-1.5">
                <span className={cn(TEXT_KICKER, "text-muted-foreground")}>
                    Powers <span className="font-normal normal-case opacity-70">(peak · sustained 24/7)</span>
                </span>
                <div className="flex flex-wrap gap-1">
                    {plan.consumers.map((c) => {
                        const a = roomAccent(c.room_type);
                        return (
                            <MiniChip
                                key={c.operator.operator_id}
                                op={c.operator}
                                suffix={
                                    <span className="font-mono text-[9px]">
                                        <span style={{ color: a.strong }}>+{c.bonus_pct.toFixed(0)}%</span> <span className="text-muted-foreground/60">+{c.sustained_pct.toFixed(0)}%·24/7</span>
                                    </span>
                                }
                                tip={
                                    <p>
                                        {c.operator.name} gains +{c.bonus_pct.toFixed(0)}% at peak ({c.sustained_pct.toFixed(0)}% sustained 24/7) from the {roomLabel(c.room_type)} pool.
                                    </p>
                                }
                            />
                        );
                    })}
                </div>
            </div>
            {plan.rotation_manager && (
                <div className="flex flex-wrap items-center gap-1.5">
                    <span className={cn(TEXT_KICKER, "text-muted-foreground")}>Rotation manager</span>
                    <MiniChip op={plan.rotation_manager} suffix={<span className="font-mono text-[9px] text-muted-foreground/70">swaps morale</span>} tip={<p>{plan.rotation_manager.name} keeps the Ling/Dusk morale rotation running.</p>} />
                </div>
            )}
            {plan.needs_rotation_manager && (
                <p className={cn(TEXT_META, "rounded-md border border-amber-500/40 bg-amber-500/10 px-2 py-1.5 text-amber-500/90")}>
                    ⚠ This rotation relies on Ling/Dusk, which need a morale-swap operator (e.g. <span className="font-medium">Fiammetta</span>) to sustain. You don't have one - the figures above are a peak ceiling rather than a sustainable plan.
                </p>
            )}
        </>
    );
}

// ─── Shared collapsible ───────────────────────────────────────────────────────

/** A collapsible secondary section (sustained rotation, peak, economy) - keeps the
 *  dialog focused on the primary view while leaving the detail one click away. */
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

// ─── Sustained 24/7 rotation (secondary) ──────────────────────────────────────

function RotationBody({ rotation, accent }: { rotation: IRotation; accent: string }) {
    return (
        <>
            <p className={cn(TEXT_META, "text-muted-foreground")}>
                Staggered rotation: keep the main team working, and when you log in swap only the <span className="font-medium">⚡ first operator</span> in each room (the one whose morale runs low soonest) for its backup - never a whole team at once. Times are roughly how long each operator works before it needs rest.
            </p>
            <div className="grid grid-cols-1 gap-1.5 sm:grid-cols-2">
                {rotation.rooms.map((room) => (
                    <RotationRoomBlock key={room.slot_id} room={room} accent={accent} />
                ))}
            </div>
            <RotationSetsSection sets={rotation.sets} accent={accent} />
        </>
    );
}

/** One production room's rotation plan as a room block: who to swap first (⚡), how long
 *  each lasts, and the dashed backup. */
function RotationRoomBlock({ room, accent }: { room: IRoomRotation; accent: string }) {
    return (
        <RoomBlock roomType={room.room_type}>
            <div className="flex flex-wrap items-center gap-0.5">
                {room.members.map((m, i) => (
                    <MiniChip
                        key={m.operator.operator_id}
                        op={m.operator}
                        lead={i === 0 ? <span className="text-[10px] text-amber-500/90">⚡</span> : undefined}
                        suffix={<span className="font-mono text-[9px] text-muted-foreground/65 tabular-nums">~{m.lasts_hours.toFixed(0)}h</span>}
                        tip={
                            <p>
                                {i === 0 ? "Swap this one first - " : ""}
                                {m.operator.name} works ~{m.lasts_hours.toFixed(0)}h before its morale runs low.
                            </p>
                        }
                    />
                ))}
                {room.backup && (
                    <>
                        <span className="px-0.5 text-[10px]" style={{ color: `color-mix(in oklch, ${accent} 55%, var(--muted-foreground))` }}>
                            ⇄
                        </span>
                        <MiniChip op={room.backup} dashed suffix={<span className="font-mono text-[9px] text-muted-foreground/55">backup</span>} tip={<p>Rotate {room.backup.name} in when the ⚡ first operator needs rest.</p>} />
                    </>
                )}
            </div>
        </RoomBlock>
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
            <p className={cn(TEXT_META, "text-muted-foreground")}>Cycle through these {sets.length} sets one swap at a time - each rests only the operators that actually run low on morale (covered by a backup). Consecutive sets share all but one operator per room.</p>
            <div className="grid grid-cols-1 gap-1.5 sm:grid-cols-2">
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
        <div className="flex flex-col gap-1 rounded-md border border-border/35 bg-muted/15 px-2 py-1.5">
            <div className="flex items-center justify-between gap-2">
                <span className={cn(TEXT_KICKER, "font-semibold")} style={{ color: accent }}>
                    Set {index + 1}
                </span>
                {resting.length > 0 && <span className={cn(TEXT_BADGE, "truncate text-muted-foreground/60")}>resting: {resting.join(", ")}</span>}
            </div>
            <div className="flex flex-col gap-1">
                {set.rooms.map((r) => (
                    <RoomBlock key={r.slot_id} roomType={r.room_type}>
                        <div className="flex flex-wrap items-center gap-0.5">
                            {r.working.map((op) => (
                                <MiniChip key={op.operator_id} op={op} />
                            ))}
                        </div>
                    </RoomBlock>
                ))}
            </div>
        </div>
    );
}

// ─── Peak single-shift (primary fallback / secondary) ─────────────────────────

const PRODUCTION_ROOMS = new Set(["MANUFACTURE", "TRADING", "POWER"]);

/** The peak arrangement as a 2-column grid of room blocks (mirrors the export's
 *  no-rotation fallback): the best single-snapshot staffing per room. */
function PeakGrid({ optimal }: { optimal: IBaseAssignment }) {
    return (
        <div className="flex flex-col gap-2.5">
            <div className="flex items-center justify-between rounded-md border border-border/35 bg-muted/10 px-2.5 py-1.5">
                <span className={cn(TEXT_KICKER, "text-muted-foreground")}>Peak yield</span>
                <YieldSummary assignment={optimal} />
            </div>
            <div className="grid grid-cols-1 gap-1.5 sm:grid-cols-2">
                {optimal.rooms.map((room) => (
                    <PeakRoomBlock key={`${room.slot_id}-${room.room_type}`} room={room} />
                ))}
            </div>
        </div>
    );
}

function PeakRoomBlock({ room }: { room: IRoomAssignment }) {
    const a = roomAccent(room.room_type);
    const yieldLabel = roomYieldLabel(room);
    // A perception-support room (a generator reserved into an Office/dormitory to feed the
    // economy) carries no efficiency of its own - show its role instead of a misleading +0%.
    // The Office/Reception staffed for their own base skill DO have a value, so show that.
    const isSupportRoom = room.room_type !== "CONTROL" && !PRODUCTION_ROOMS.has(room.room_type) && room.total_efficiency === 0;
    const right = isSupportRoom ? (
        <span className="text-[9px] text-muted-foreground/70">economy</span>
    ) : (
        <span className="font-mono font-semibold text-[10px] tabular-nums" style={{ color: a.strong }}>
            +{room.total_efficiency.toFixed(0)}%
        </span>
    );
    const badge =
        room.locked && room.operators.length > 0 ? (
            <Tooltip>
                <TooltipTrigger render={<span className="cursor-help text-amber-500/80">🔒</span>} />
                <TooltipContent sideOffset={4}>
                    <p>Fixed synergy squad - these operators depend on each other and can't be swapped without breaking the combo.</p>
                </TooltipContent>
            </Tooltip>
        ) : undefined;
    return (
        <RoomBlock roomType={room.room_type} formula={room.formula_type} badge={badge} right={right}>
            <div className="flex flex-wrap items-center gap-0.5">
                {room.operators.length === 0 && <span className="text-[10px] text-muted-foreground/50">No assignment</span>}
                {room.operators.map((op) => (
                    <MiniChip key={op.operator_id} op={op} tip={<p>{op.name}</p>} />
                ))}
            </div>
            {(room.order_value > 0 || yieldLabel || room.room_type === "CONTROL") && (
                <div className="flex flex-wrap items-center gap-x-1.5 font-mono text-[9px] text-muted-foreground/65">
                    {room.room_type === "CONTROL" && <span>global buff</span>}
                    {room.order_value > 0 && <span className="text-amber-500/85">+{room.order_value.toFixed(0)}% value</span>}
                    {yieldLabel && <span>{yieldLabel}</span>}
                </div>
            )}
        </RoomBlock>
    );
}
