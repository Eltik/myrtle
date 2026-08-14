import type { IBaseImprovements } from "#/lib/api/user";
import { PeakRoomGrid } from "./PeakSection";
import { roomLabel } from "./roomColors";
import { ShiftGrid } from "./ShiftSection";
import { compactNum } from "./yield";

/**
 * A compact, fixed-width, multi-column layout of the base plan tailored for image
 * export - the three shifts sit side by side so the result is a wide poster rather
 * than the tall single-column strip the dialog renders. It reuses the SAME grid
 * components as the dialog (with interactivity off), so the PNG always matches
 * what's on screen.
 */
export function ExportPlanContent({ base }: { base: IBaseImprovements }) {
    const rotation = base.shift_rotation;
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
                    <ShiftGrid rotation={rotation} className="grid-cols-3" />
                </>
            ) : (
                base.optimal && <PeakRoomGrid optimal={base.optimal} className="grid-cols-2" />
            )}
        </div>
    );
}
