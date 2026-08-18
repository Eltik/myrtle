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

export function crewsForShift(api: IOptimizerAPI, presetBySlot: Map<string, string[][]>): IShiftCrews {
    const shift = api.viewShift;
    if (shift === null) return { rooms: api.boardRooms };

    if (api.rotation === null) {
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
