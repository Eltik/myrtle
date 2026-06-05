import type { ReactNode } from "react";
import { OperatorAvatar } from "#/components/ui/operator-avatar";
import { Tooltip, TooltipContent, TooltipTrigger } from "#/components/ui/tooltip";
import type { IAssignedOperator, IShiftRoom } from "#/lib/api/user";
import { cn } from "#/lib/utils";
import { roomAccent, roomFormulaLabel } from "./roomColors";

// Shared building blocks for the base plan, used by BOTH the in-app dialog and the
// downloadable poster (ExportPlanContent) so the two read identically: a compact,
// colour-accented, poster-style layout rather than a tall verbose list.

export type ChipTone = "add" | "remove" | "keep";

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
            <span className={cn("max-w-[8ch] truncate font-medium text-[10px]", strike && "text-rose-500/80 line-through", dashed && "text-muted-foreground")}>{op.name}</span>
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
            <span className="flex items-center gap-1 font-medium text-[10px]" style={{ color: a.text }}>
                <span className="size-1.5 shrink-0 rounded-xs" style={{ background: a.color }} />
                <span className="truncate">{roomFormulaLabel(roomType, formula)}</span>
                {badge}
                {right && <span className="ml-auto shrink-0 pl-1">{right}</span>}
            </span>
            {children}
        </div>
    );
}

/**
 * One room within a shift, rendered identically in the in-app dialog and the downloadable
 * poster so the export always matches what you see. Shows the crew to run this shift (green =
 * add vs your preset), an "out" row for operators to pull, and the off / matches / "≈ yours"
 * (your team already ties the recommendation) states. `interactive` adds hover tooltips for the
 * dialog; the export passes it off so html-to-image captures a clean, static image.
 */
export function ShiftRoomBlock({ room, interactive = false }: { room: IShiftRoom; interactive?: boolean }) {
    const swapIn = new Set(room.swap_in.map((o) => o.operator_id));
    const hasPreset = room.current.length > 0;
    // A tie on output: show the player's OWN team (keep it) rather than nudging a pointless swap.
    const isEquivalent = room.active && room.equivalent;
    const crew = isEquivalent ? room.current : room.recommended;
    const equivalentBadge = interactive ? (
        <Tooltip>
            <TooltipTrigger render={<span className="cursor-help text-emerald-500/80">· ≈ yours</span>} />
            <TooltipContent sideOffset={4}>
                <p className="max-w-xs">Your current team here already produces as much as the recommended one, so there's no need to swap.</p>
            </TooltipContent>
        </Tooltip>
    ) : (
        <span className="text-emerald-500/80">· ≈ yours</span>
    );
    const badge = !room.active ? <span className="text-amber-500/80">· off</span> : isEquivalent ? equivalentBadge : hasPreset && room.matches ? <span className="text-emerald-500/80">· ✓</span> : undefined;
    const showRemoved = room.active && !isEquivalent && hasPreset && !room.matches && room.swap_out.length > 0;
    return (
        <RoomBlock roomType={room.room_type} formula={room.formula_type} dim={!room.active} badge={badge}>
            <div className="flex flex-wrap items-center gap-0.5">
                {crew.length === 0 && <span className="text-[10px] text-muted-foreground/50">-</span>}
                {crew.map((op) => {
                    const add = !isEquivalent && swapIn.has(op.operator_id);
                    return (
                        <MiniChip
                            key={op.operator_id}
                            op={op}
                            tone={add ? "add" : "keep"}
                            tip={
                                interactive ? (
                                    <p>
                                        {add ? "Add to this shift - " : ""}
                                        {op.name}
                                    </p>
                                ) : undefined
                            }
                        />
                    );
                })}
            </div>
            {showRemoved && (
                <div className="flex flex-wrap items-center gap-0.5">
                    <span className="mr-0.5 font-mono text-[9px] text-muted-foreground/50 uppercase tracking-wide">out</span>
                    {room.swap_out.map((op) => (
                        <MiniChip key={op.operator_id} op={op} tone="remove" strike tip={interactive ? <p>Pull {op.name} from this shift.</p> : undefined} />
                    ))}
                </div>
            )}
            {!hasPreset && room.active && <span className="text-[9px] text-muted-foreground/45">no saved preset</span>}
        </RoomBlock>
    );
}
