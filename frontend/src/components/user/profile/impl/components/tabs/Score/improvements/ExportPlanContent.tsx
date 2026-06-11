import type { IBaseImprovements } from "#/lib/api/user";
import { compactNum } from "./baseYield";
import { MiniChip, RoomBlock, ShiftRoomBlock } from "./planParts";
import { roomLabel } from "./roomColors";

/** A room with its own base-skill efficiency (production, Control Center, a staffed
 *  Office/Reception) shows a "+X%"; a perception-support room reserved to feed the economy
 *  carries no efficiency, so it shows an "economy" tag instead of a misleading +0%. */
const PRODUCTION_OR_CONTROL = new Set(["MANUFACTURE", "TRADING", "POWER", "CONTROL"]);

/**
 * A compact, fixed-width, multi-column layout of the base plan tailored for image
 * export - the three shifts sit side by side so the result is a wide poster rather
 * than the tall single-column strip the dialog renders. Rooms carry their real
 * in-game colours, matching the dialog.
 */
export function ExportPlanContent({ base }: { base: IBaseImprovements }) {
    const rotation = base.shift_rotation;
    const sustainedSet = new Set(rotation?.sustained.map((o) => o.operator_id) ?? []);
    return (
        <div className="flex w-240 flex-col gap-3 bg-background p-5 text-foreground">
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
                                    <ShiftRoomBlock key={room.slot_id} room={room} sustained={sustainedSet} />
                                ))}
                            </div>
                        ))}
                    </div>
                </>
            ) : (
                base.optimal && (
                    <div className="grid grid-cols-2 gap-2.5">
                        {base.optimal.rooms.map((room) => (
                            <RoomBlock
                                key={`${room.slot_id}-${room.room_type}`}
                                roomType={room.room_type}
                                formula={room.formula_type}
                                right={PRODUCTION_OR_CONTROL.has(room.room_type) || room.total_efficiency > 0 ? <span className="font-mono text-[10px]">+{room.total_efficiency.toFixed(0)}%</span> : <span className="text-muted-foreground/70">economy</span>}
                            >
                                <div className="flex flex-wrap items-center gap-0.5">
                                    {room.operators.map((op) => (
                                        <MiniChip key={op.operator_id} op={op} />
                                    ))}
                                </div>
                            </RoomBlock>
                        ))}
                    </div>
                )
            )}
        </div>
    );
}
