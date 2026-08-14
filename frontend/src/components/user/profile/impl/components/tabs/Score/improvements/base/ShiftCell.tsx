import { Tooltip, TooltipContent, TooltipTrigger } from "#/components/ui/tooltip";
import type { IAssignedOperator, IShiftRoom } from "#/lib/api/user";
import { cn } from "#/lib/utils";
import { MiniChip, RoomBlock, RowKicker, TEXT_CHIP, TEXT_MICRO, TEXT_TINY } from "./parts";
import { roomFormulaLabel } from "./roomColors";

/** "−3%" style note for the leniency gap; hidden for negligible gaps. */
function gapNote(gapPct: number | null | undefined): string | null {
    if (gapPct == null || Math.abs(gapPct) < 0.5) return null;
    const rounded = Math.round(gapPct * 10) / 10;
    return `${rounded > 0 ? "+" : "−"}${Math.abs(rounded).toFixed(Math.abs(rounded) >= 10 ? 0 : 1)}%`;
}

/** Everything the cell renders, derived once from the payload so the render
 *  below is a plain readout instead of inline set algebra. */
function deriveCell(room: IShiftRoom) {
    const swapIn = new Set(room.swap_in.map((o) => o.operator_id));
    const hasPreset = room.current.length > 0;
    // Within the leniency band: show the player's OWN team (keep it) rather than
    // nudging a pointless swap, with the small gap surfaced on the badge.
    const isEquivalent = room.active && room.equivalent;
    const crew = isEquivalent ? room.current : room.recommended;
    const gap = gapNote(room.gap_pct);
    const gapBehind = gap !== null && (room.gap_pct ?? 0) < 0;
    const showRemoved = room.active && !isEquivalent && hasPreset && !room.matches && room.swap_out.length > 0;
    // Only ≈-yours cells show the side-by-side: the player's team above, the
    // system's recommendation in a muted "rec" row below (when they differ).
    const recSet = new Set(room.recommended.map((o) => o.operator_id));
    const curSet = new Set(room.current.map((o) => o.operator_id));
    const showRecRow = isEquivalent && (room.recommended.length !== room.current.length || room.recommended.some((o) => !curSet.has(o.operator_id)) || room.current.some((o) => !recSet.has(o.operator_id)));
    const showMoves = room.active && !isEquivalent && (room.moved_out?.length ?? 0) > 0;
    return { swapIn, hasPreset, isEquivalent, crew, gap, gapBehind, showRemoved, showRecRow, showMoves };
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
            {room.efficiency != null && room.active && <span className={cn("font-mono text-muted-foreground/70", TEXT_MICRO)}>{Math.round(room.efficiency)}%</span>}
            <span className={cn("rounded-sm px-1 font-semibold leading-tight", TEXT_MICRO)} style={{ background: `hsl(${hue} 70% 50% / 0.18)`, color: `hsl(${hue} 75% 65%)` }}>
                {room.team_label}
            </span>
        </span>
    );
}

/** The "· off" / "· ✓" / "· ≈ yours (−3%)" state marker beside the room label. */
function CellBadge({ room, cell, interactive }: { room: IShiftRoom; cell: ReturnType<typeof deriveCell>; interactive: boolean }) {
    if (!room.active) return <span className="text-amber-500/80">· off</span>;
    if (cell.isEquivalent) {
        const label = cell.gapBehind ? `· ≈ yours (${cell.gap})` : "· ≈ yours";
        if (!interactive) return <span className="text-emerald-500/80">{label}</span>;
        return (
            <Tooltip>
                <TooltipTrigger render={<span className="cursor-help text-emerald-500/80">{label}</span>} />
                <TooltipContent sideOffset={4}>
                    <p className="max-w-xs">
                        Your current team here is close enough to the recommended one that swapping isn't worth it.
                        {cell.gapBehind ? ` The recommendation below would produce about ${cell.gap?.replace("−", "")} more.` : ""}
                    </p>
                </TooltipContent>
            </Tooltip>
        );
    }
    if (cell.hasPreset && room.matches) return <span className="text-emerald-500/80">· ✓</span>;
    return null;
}

function SustainedBadge() {
    return <span className={cn("rounded-sm bg-amber-500/20 px-0.5 font-semibold text-amber-500/90 leading-none", TEXT_MICRO)}>24/7</span>;
}

function CrewRow({ cell, interactive, sustained }: { cell: ReturnType<typeof deriveCell>; interactive: boolean; sustained?: Set<string> }) {
    return (
        <div className="flex flex-wrap items-center gap-0.5">
            {cell.crew.length === 0 && <span className={cn("text-muted-foreground/50", TEXT_CHIP)}>-</span>}
            {cell.crew.map((op: IAssignedOperator) => {
                const add = !cell.isEquivalent && cell.swapIn.has(op.operator_id);
                const is247 = sustained?.has(op.operator_id) ?? false;
                return (
                    <MiniChip
                        key={op.operator_id}
                        op={op}
                        tone={add ? "add" : "keep"}
                        suffix={is247 ? <SustainedBadge /> : undefined}
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
    );
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
    const cell = deriveCell(room);
    return (
        <RoomBlock roomType={room.room_type} formula={room.formula_type} dim={!room.active} badge={<CellBadge room={room} cell={cell} interactive={interactive} />} right={<TeamTag room={room} />}>
            <CrewRow cell={cell} interactive={interactive} sustained={sustained} />
            {cell.showRecRow && (
                <div className="flex flex-wrap items-center gap-0.5 opacity-70">
                    <RowKicker>rec</RowKicker>
                    {room.recommended.map((op) => (
                        <MiniChip key={op.operator_id} op={op} dashed tip={interactive ? <p>The system's pick: {op.name}.</p> : undefined} />
                    ))}
                </div>
            )}
            {cell.showRemoved && (
                <div className="flex flex-wrap items-center gap-0.5">
                    <RowKicker>out</RowKicker>
                    {room.swap_out.map((op) => (
                        <MiniChip key={op.operator_id} op={op} tone="remove" strike tip={interactive ? <p>Pull {op.name} from this shift.</p> : undefined} />
                    ))}
                </div>
            )}
            {cell.showMoves && (
                <div className="flex flex-wrap items-center gap-0.5">
                    <RowKicker>moves</RowKicker>
                    {room.moved_out?.map((m) => (
                        <MiniChip
                            key={m.operator.operator_id}
                            op={m.operator}
                            dashed
                            suffix={
                                <span className={cn("text-muted-foreground/70", TEXT_MICRO)}>
                                    → {roomFormulaLabel(m.to_room_type, null)}
                                    {m.to_team_label ? ` · ${m.to_team_label}` : ""}
                                </span>
                            }
                            tip={
                                interactive ? (
                                    <p>
                                        {m.operator.name} isn't benched - they move to {roomFormulaLabel(m.to_room_type, null)}
                                        {m.to_team_label ? ` (${m.to_team_label})` : ""} this shift.
                                    </p>
                                ) : undefined
                            }
                        />
                    ))}
                </div>
            )}
            {!cell.hasPreset && room.active && <span className={cn("text-muted-foreground/45", TEXT_TINY)}>no saved preset</span>}
        </RoomBlock>
    );
}
