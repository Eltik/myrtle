import type { IDraftRoom } from "#/lib/api/base";
import type { BoardMarks } from "#/lib/base/board";
import type { IOptimizerAPI } from "#/lib/base/use-optimizer";

function presetForShift(shifts: string[][], shiftIndex: number): string[] {
    const filled = shifts.filter((crew) => crew.length > 0);
    if (filled.length === 0) return [];
    return filled[shiftIndex % filled.length];
}

interface IShiftCrews {
    rooms: IDraftRoom[];
    marks?: BoardMarks;
}

/** Same operators, order aside. */
export function sameCrew(a: readonly string[], b: readonly string[]): boolean {
    if (a.length !== b.length) return false;
    const want = new Set(a);
    return b.every((id) => want.has(id));
}

/**
 * Whose figures a tile's popover may show. The board displays the draft (no shift
 * tab), the rotation's cell (a shift tab with a plan), or the player's own preset
 * crew (a shift tab before a plan exists, see `crewsForShift`). The ledger and
 * efficiency must describe that same crew: a preset crew has no scored cell of its
 * own, so it reads the draft's figures only when it IS the draft crew, else nothing.
 * Reading the rotation's cell for a crew it did not score showed Amiya's Control
 * Center bonus over a player's Trading Post crew whose Control Center held no Amiya.
 */
export function crewSource(viewShift: number | null, hasPlan: boolean, tileIds: readonly string[], scoredIds: readonly string[] | undefined): "draft" | "plan" | "preset" {
    if (viewShift === null) return "draft";
    if (hasPlan) return "plan";
    return scoredIds !== undefined && sameCrew(tileIds, scoredIds) ? "draft" : "preset";
}

export function crewsForShift(api: IOptimizerAPI, presetBySlot: Map<string, string[][]>): IShiftCrews {
    const shift = api.viewShift;
    if (shift === null) return { rooms: api.boardRooms };

    if (api.proposal === null || api.rotation === null) {
        return { rooms: api.boardRooms.map((room) => ({ ...room, operators: presetForShift(presetBySlot.get(room.slot_id) ?? [], shift - 1) })) };
    }

    const marks: BoardMarks = new Map();
    const rooms = api.boardRooms.map((room) => {
        const plan = api.shiftRoom(room.slot_id);
        if (!plan) return room;

        const added = new Set(plan.swap_in.map((op) => op.operator_id));
        const removed = plan.swap_out.map((op) => op.operator_id);
        const crew = plan.recommended.map((op) => op.operator_id);

        const slotMarks = new Map<string, "added" | "removed">();
        for (const id of crew) if (added.has(id)) slotMarks.set(id, "added");
        for (const id of removed) slotMarks.set(id, "removed");
        if (slotMarks.size > 0) marks.set(room.slot_id, slotMarks);

        return { ...room, operators: [...crew, ...removed] };
    });

    return { rooms, marks };
}
