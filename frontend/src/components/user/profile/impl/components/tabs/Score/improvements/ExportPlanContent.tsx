import { OperatorAvatar } from "#/components/ui/operator-avatar";
import type { IAssignedOperator, IBaseImprovements, IShiftRoom } from "#/lib/api/user";
import { cn } from "#/lib/utils";
import { compactNum } from "./baseYield";
import { roomAccent, roomFormulaLabel, roomLabel } from "./roomColors";

function OpChip({ op, add }: { op: IAssignedOperator; add?: boolean }) {
    return (
        <span className={cn("flex items-center gap-1 rounded border px-1 py-0.5", add ? "border-emerald-500/55 bg-emerald-500/10" : "border-border/40 bg-background/60")}>
            <span className="relative size-4 shrink-0 overflow-hidden rounded-sm">
                <OperatorAvatar charId={op.operator_id} name={op.name} />
            </span>
            <span className="max-w-[8ch] truncate font-medium text-[10px]">{op.name}</span>
        </span>
    );
}

/** A room's crew with a colour accent matching the building's in-game hue. */
function ShiftRoomBlock({ room }: { room: IShiftRoom }) {
    const a = roomAccent(room.room_type);
    const swapIn = new Set(room.swap_in.map((o) => o.operator_id));
    return (
        <div className={cn("flex flex-col gap-0.5 rounded-r border-l-2 pl-1.5", !room.active && "opacity-60")} style={{ borderColor: a.border, background: a.tint }}>
            <span className="flex items-center gap-1 font-medium text-[10px]" style={{ color: a.text }}>
                <span className="size-1.5 rounded-[2px]" style={{ background: a.color }} />
                {roomFormulaLabel(room.room_type, room.formula_type)}
                {!room.active && <span className="text-amber-500/80"> · off</span>}
            </span>
            <div className="flex flex-wrap items-center gap-0.5">
                {room.recommended.map((op) => (
                    <OpChip key={op.operator_id} op={op} add={swapIn.has(op.operator_id)} />
                ))}
            </div>
        </div>
    );
}

/**
 * A compact, fixed-width, multi-column layout of the base plan tailored for image
 * export - the three shifts sit side by side so the result is a wide poster rather
 * than the tall single-column strip the dialog renders. Rooms carry their real
 * in-game colours, matching the dialog.
 */
export function ExportPlanContent({ base }: { base: IBaseImprovements }) {
    const rotation = base.shift_rotation;
    return (
        <div className="flex w-[960px] flex-col gap-3 bg-background p-5 text-foreground">
            <div className="flex items-end justify-between border-border/40 border-b pb-2">
                <div className="flex flex-col">
                    <span className="font-semibold text-base">Base optimization plan</span>
                    <span className="text-[11px] text-muted-foreground">{base.layout.map((l) => `${l.count}× ${roomLabel(l.room_type)}`).join("  ·  ")}</span>
                </div>
                <div className="flex flex-col items-end">
                    {base.optimal && (
                        <span className="font-mono font-semibold text-sm tabular-nums">
                            {compactNum(base.optimal.yield_lmd_per_day)} LMD
                            {base.optimal.yield_exp_per_day > 0 && <> · {compactNum(base.optimal.yield_exp_per_day)} EXP</>}
                            <span className="text-[10px] text-muted-foreground"> /day avg</span>
                        </span>
                    )}
                    <span className="text-[10px] text-muted-foreground">myrtle.moe</span>
                </div>
            </div>

            {rotation && rotation.shifts.length > 0 ? (
                <>
                    <span className="font-medium text-[11px] text-muted-foreground">Three-shift rotation - each team rests one shift (green = add vs your preset).</span>
                    <div className="grid grid-cols-3 gap-2.5">
                        {rotation.shifts.map((shift) => (
                            <div key={shift.index} className="flex flex-col gap-1.5 rounded-md border border-border/35 bg-muted/10 p-2">
                                <span className="font-semibold text-[11px]">Shift {shift.index}</span>
                                {shift.rooms.map((room) => (
                                    <ShiftRoomBlock key={room.slot_id} room={room} />
                                ))}
                            </div>
                        ))}
                    </div>
                </>
            ) : (
                base.optimal && (
                    <div className="grid grid-cols-2 gap-2.5">
                        {base.optimal.rooms.map((room) => {
                            const a = roomAccent(room.room_type);
                            return (
                                <div key={`${room.slot_id}-${room.room_type}`} className="flex flex-col gap-0.5 rounded-md border-2 p-2" style={{ borderColor: a.border, background: a.tint }}>
                                    <span className="flex items-center gap-1 font-medium text-[10px]" style={{ color: a.text }}>
                                        <span className="size-1.5 rounded-[2px]" style={{ background: a.color }} />
                                        {roomFormulaLabel(room.room_type, room.formula_type)} · +{room.total_efficiency.toFixed(0)}%
                                    </span>
                                    <div className="flex flex-wrap items-center gap-0.5">
                                        {room.operators.map((op) => (
                                            <OpChip key={op.operator_id} op={op} />
                                        ))}
                                    </div>
                                </div>
                            );
                        })}
                    </div>
                )
            )}
        </div>
    );
}
