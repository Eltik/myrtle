import type { ReactNode } from "react";
import { OperatorAvatar } from "#/components/ui/operator-avatar";
import { Tooltip, TooltipContent, TooltipTrigger } from "#/components/ui/tooltip";
import type { IAssignedOperator, IBaseAssignment, IRoomLayoutEntry } from "#/lib/api/user";
import { cn } from "#/lib/utils";
import { TEXT_BADGE, TEXT_KICKER } from "../shared";
import { roomAccent, roomFormulaLabel, roomLabel } from "./roomColors";
import { compactNum, signedCompact } from "./yield";

// Shared building blocks for the base plan, used by BOTH the in-app dialog and the
// downloadable poster (ExportPlanContent) so the two read identically: a compact,
// colour-accented, poster-style layout rather than a tall verbose list.

// ─── Poster typography ────────────────────────────────────────────────────────
// The poster is one density step below the panel scale in ../shared.tsx; these
// are its only sizes. New poster sizes extend THIS list - they don't go inline.

/** Chip labels, room titles, small notes. */
export const TEXT_CHIP = "text-[10px]";
/** Suffix/metric annotations riding on a chip. */
export const TEXT_TINY = "text-[9px]";
/** The smallest marks: team tags, 24/7 badges, inline % figures. */
export const TEXT_MICRO = "text-[8px]";

type ChipTone = "add" | "remove" | "keep";

const TONE: Record<ChipTone, string> = {
    add: "border-emerald-500/55 bg-emerald-500/12",
    remove: "border-rose-500/45 bg-rose-500/12",
    keep: "border-border/40 bg-background/60",
};

/**
 * Compact operator chip - the atom of the poster layout. `tone` colours it for the
 * preset diff (green add / red remove), `lead`/`suffix` add small markers (a swap-first
 * bolt, a "~6h" lifetime), `dashed` marks a backup, and `tip` wraps it in a tooltip for
 * the interactive dialog. The export passes no `tip`, so it renders a plain chip.
 */
export function MiniChip({ op, tone = "keep", strike, dashed, lead, suffix, tip }: { op: IAssignedOperator; tone?: ChipTone; strike?: boolean; dashed?: boolean; lead?: ReactNode; suffix?: ReactNode; tip?: ReactNode }) {
    const chip = (
        <span className={cn("flex items-center gap-1 rounded border px-1 py-0.5", dashed ? "border-border/45 border-dashed bg-background/40" : TONE[tone])}>
            {lead}
            <span className={cn("relative size-4 shrink-0 overflow-hidden rounded-sm", dashed && "opacity-80")}>
                <OperatorAvatar charId={op.operator_id} name={op.name} />
            </span>
            <span className={cn("max-w-[8ch] truncate font-medium", TEXT_CHIP, strike && "text-rose-500/80 line-through", dashed && "text-muted-foreground")}>{op.name}</span>
            {suffix}
        </span>
    );
    if (!tip) return chip;
    return (
        <Tooltip>
            <TooltipTrigger render={chip} />
            <TooltipContent sideOffset={4}>{tip}</TooltipContent>
        </Tooltip>
    );
}

/**
 * A room block: a thin left-border accent strip in the building's in-game colour, with
 * the room label/formula on top and a wrapped chip row below. `badge` sits next to the
 * label (off/✓), `right` is pushed to the far end (an efficiency figure or tag).
 */
export function RoomBlock({ roomType, formula, badge, right, dim, children }: { roomType: string; formula?: string | null; badge?: ReactNode; right?: ReactNode; dim?: boolean; children: ReactNode }) {
    const a = roomAccent(roomType);
    return (
        <div className={cn("flex flex-col gap-0.5 rounded-r border-l-2 py-0.5 pr-1 pl-1.5", dim && "opacity-60")} style={{ borderColor: a.border, background: a.tint }}>
            <span className={cn("flex items-center gap-1 font-medium", TEXT_CHIP)} style={{ color: a.text }}>
                <span className="size-1.5 shrink-0 rounded-xs" style={{ background: a.color }} />
                <span className="truncate">{roomFormulaLabel(roomType, formula)}</span>
                {badge}
                {right && <span className="ml-auto shrink-0 pl-1">{right}</span>}
            </span>
            {children}
        </div>
    );
}

// ─── Layout chips ─────────────────────────────────────────────────────────────

/**
 * The base layout as colour chips - one per room type, tinted with the in-game
 * room hue, doubling as the plan's colour legend. `detailed` adds the level list
 * (the collapsed panel view); the compact form headers the plan dialog.
 */
export function LayoutChips({ layout, detailed }: { layout: IRoomLayoutEntry[]; detailed?: boolean }) {
    if (layout.length === 0) return null;
    return (
        <div className="flex flex-wrap items-center gap-1.5">
            {layout.map((l) => {
                const a = roomAccent(l.room_type);
                return (
                    <span key={l.room_type} className={cn("flex items-center gap-1 rounded border", detailed ? "gap-1.5 rounded-md px-2 py-1 text-[11.5px]" : cn("px-1.5 py-0.5", TEXT_CHIP))} style={{ borderColor: a.border, background: a.tint }}>
                        <span className={cn("rounded-xs", detailed ? "size-2 rounded-[3px]" : "size-1.5")} style={{ background: a.color }} />
                        <span className="font-medium" style={{ color: a.text }}>
                            {roomLabel(l.room_type)}
                        </span>
                        <span className={cn(detailed && TEXT_BADGE, "text-muted-foreground")}>×{l.count}</span>
                        {detailed && <span className={cn(TEXT_BADGE, "text-muted-foreground")}>L{l.levels.join(", L")}</span>}
                    </span>
                );
            })}
        </div>
    );
}

// ─── Yield figures ────────────────────────────────────────────────────────────

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
export function YieldSummary({ assignment }: { assignment: IBaseAssignment }) {
    return (
        <span className={cn(TEXT_BADGE, "text-muted-foreground")}>
            {compactNum(assignment.yield_lmd_per_day)} LMD
            {assignment.yield_exp_per_day > 0 && <> · {compactNum(assignment.yield_exp_per_day)} EXP</>}
            <PerDay className="opacity-60" />
        </span>
    );
}

/** Current → optimal daily-yield line: the realized current yield, the optimal LMD/EXP,
 *  and the efficiency delta between them. Accent flows from `--imp-accent`. */
export function YieldHeadline({ current, optimal }: { current: IBaseAssignment; optimal: IBaseAssignment }) {
    return (
        <div className="flex flex-wrap items-baseline gap-x-2 gap-y-1">
            <YieldSummary assignment={current} />
            <span className="text-muted-foreground/50">→</span>
            <span className="font-mono font-semibold text-sm tabular-nums" style={{ color: "color-mix(in oklch, var(--imp-accent) 70%, var(--foreground))" }}>
                {compactNum(optimal.yield_lmd_per_day)} LMD
                {optimal.yield_exp_per_day > 0 && <> · {compactNum(optimal.yield_exp_per_day)} EXP</>}
                <PerDay className="font-sans text-muted-foreground/60" />
            </span>
            <span className={cn(TEXT_BADGE, "text-muted-foreground")}>({signedCompact(optimal.total_production_efficiency - current.total_production_efficiency)}% efficiency)</span>
        </div>
    );
}

// ─── Small labels ─────────────────────────────────────────────────────────────

/** An uppercase row label inside a poster cell ("rec" / "out" / "moves"). */
export function RowKicker({ children }: { children: ReactNode }) {
    return <span className={cn("mr-0.5 font-mono text-muted-foreground/50 uppercase tracking-wide", TEXT_TINY)}>{children}</span>;
}

/** A kicker coloured by the panel accent, without threading the accent prop. */
export function AccentKicker({ children }: { children: ReactNode }) {
    return (
        <span className={cn(TEXT_KICKER)} style={{ color: "color-mix(in oklch, var(--imp-accent) 60%, var(--foreground))" }}>
            {children}
        </span>
    );
}
