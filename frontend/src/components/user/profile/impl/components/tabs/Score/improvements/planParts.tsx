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

/** Deterministic accent hue for a team id, so the same team's cells visually pair up
 *  across the two shift columns its 24h block spans. */
function teamHue(teamId: string): number {
    let h = 0;
    for (let i = 0; i < teamId.length; i++) h = (h * 31 + teamId.charCodeAt(i)) >>> 0;
    return h % 360;
}

/** Small team/squad tag ("Team B" / "Squad 1") tinted by its stable team id, with the
 *  crew's efficiency % beside it (production/power cells) so output distribution is visible. */
function TeamTag({ room }: { room: IShiftRoom }) {
    if (!room.team_label) return null;
    const hue = teamHue(room.team_id ?? room.team_label);
    return (
        <span className="flex items-center gap-1">
            {room.efficiency != null && room.active && <span className="font-mono text-[8px] text-muted-foreground/70">{Math.round(room.efficiency)}%</span>}
            <span className="rounded-sm px-1 font-semibold text-[8px] leading-tight" style={{ background: `hsl(${hue} 70% 50% / 0.18)`, color: `hsl(${hue} 75% 65%)` }}>
                {room.team_label}
            </span>
        </span>
    );
}

/** "−3%" style note for the leniency gap; hidden for negligible gaps. */
function gapNote(gapPct: number | null | undefined): string | null {
    if (gapPct == null || Math.abs(gapPct) < 0.5) return null;
    const rounded = Math.round(gapPct * 10) / 10;
    return `${rounded > 0 ? "+" : "−"}${Math.abs(rounded).toFixed(Math.abs(rounded) >= 10 ? 0 : 1)}%`;
}

/**
 * One room within a shift, rendered identically in the in-app dialog and the downloadable
 * poster so the export always matches what you see. Shows the crew to run this shift (green =
 * add vs your preset), an "out" row for operators to pull, and the off / matches / "≈ yours"
 * (your team is within the leniency band of the recommendation) states - equivalent cells also
 * show the system's pick in a muted "rec" row so the alternative stays visible. `interactive`
 * adds hover tooltips for the dialog; the export passes it off so html-to-image captures a
 * clean, static image.
 */
export function ShiftRoomBlock({ room, interactive = false, sustained }: { room: IShiftRoom; interactive?: boolean; sustained?: Set<string> }) {
    const swapIn = new Set(room.swap_in.map((o) => o.operator_id));
    const hasPreset = room.current.length > 0;
    // Within the leniency band: show the player's OWN team (keep it) rather than nudging a
    // pointless swap, with the small gap surfaced on the badge.
    const isEquivalent = room.active && room.equivalent;
    const crew = isEquivalent ? room.current : room.recommended;
    const gap = gapNote(room.gap_pct);
    const equivalentLabel = gap && (room.gap_pct ?? 0) < 0 ? `· ≈ yours (${gap})` : "· ≈ yours";
    const equivalentBadge = interactive ? (
        <Tooltip>
            <TooltipTrigger render={<span className="cursor-help text-emerald-500/80">{equivalentLabel}</span>} />
            <TooltipContent sideOffset={4}>
                <p className="max-w-xs">
                    Your current team here is close enough to the recommended one that swapping isn't worth it.
                    {gap && (room.gap_pct ?? 0) < 0 ? ` The recommendation below would produce about ${gap.replace("−", "")} more.` : ""}
                </p>
            </TooltipContent>
        </Tooltip>
    ) : (
        <span className="text-emerald-500/80">{equivalentLabel}</span>
    );
    const badge = !room.active ? <span className="text-amber-500/80">· off</span> : isEquivalent ? equivalentBadge : hasPreset && room.matches ? <span className="text-emerald-500/80">· ✓</span> : undefined;
    const showRemoved = room.active && !isEquivalent && hasPreset && !room.matches && room.swap_out.length > 0;
    // Only ≈-yours cells show the side-by-side: the player's team above, the system's
    // recommendation in a muted "rec" row below (when they actually differ).
    const recSet = new Set(room.recommended.map((o) => o.operator_id));
    const curSet = new Set(room.current.map((o) => o.operator_id));
    const showRecRow = isEquivalent && (room.recommended.length !== room.current.length || room.recommended.some((o) => !curSet.has(o.operator_id)) || room.current.some((o) => !recSet.has(o.operator_id)));
    return (
        <RoomBlock roomType={room.room_type} formula={room.formula_type} dim={!room.active} badge={badge} right={<TeamTag room={room} />}>
            <div className="flex flex-wrap items-center gap-0.5">
                {crew.length === 0 && <span className="text-[10px] text-muted-foreground/50">-</span>}
                {crew.map((op) => {
                    const add = !isEquivalent && swapIn.has(op.operator_id);
                    const is247 = sustained?.has(op.operator_id) ?? false;
                    return (
                        <MiniChip
                            key={op.operator_id}
                            op={op}
                            tone={add ? "add" : "keep"}
                            suffix={is247 ? <span className="rounded-sm bg-amber-500/20 px-0.5 font-semibold text-[8px] text-amber-500/90 leading-none">24/7</span> : undefined}
                            tip={
                                interactive ? (
                                    <p>
                                        {add ? "Add to this shift - " : ""}
                                        {op.name}
                                        {is247 ? " · kept at full morale 24/7 by Fiammetta" : ""}
                                    </p>
                                ) : undefined
                            }
                        />
                    );
                })}
            </div>
            {showRecRow && (
                <div className="flex flex-wrap items-center gap-0.5 opacity-70">
                    <span className="mr-0.5 font-mono text-[9px] text-muted-foreground/50 uppercase tracking-wide">rec</span>
                    {room.recommended.map((op) => (
                        <MiniChip key={op.operator_id} op={op} dashed tip={interactive ? <p>The system's pick: {op.name}.</p> : undefined} />
                    ))}
                </div>
            )}
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
