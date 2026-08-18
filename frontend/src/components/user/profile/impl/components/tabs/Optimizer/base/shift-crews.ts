import type { IDraftRoom } from "#/lib/api/base";
import type { BoardMarks, Catalog } from "#/lib/base/layout";
import { seatsOf } from "#/lib/base/layout";
import type { IOptimizerApi } from "#/lib/base/use-optimizer";

const NO_PRESETS: ReadonlySet<string> = new Set(["DORMITORY", "WORKSHOP", "TRAINING"]);

function presetForShift(shifts: string[][], shiftIndex: number): string[] {
    const filled = shifts.filter((crew) => crew.length > 0);
    if (filled.length === 0) return [];
    return filled[shiftIndex % filled.length];
}

export interface IShiftCrews {
    rooms: IDraftRoom[];
    marks?: BoardMarks;
    offDuty?: number;
}

export function crewsForShift(api: IOptimizerApi, presetBySlot: Map<string, string[][]>, catalog: Catalog): IShiftCrews {
    const shift = api.viewShift;
    if (shift === null) return { rooms: api.boardRooms };

    const shiftIndex = shift - 1;
    const planned = api.rotation !== null;

    if (planned) {
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

    const working = new Set<string>();
    const rooms = api.boardRooms.map((room) => {
        if (NO_PRESETS.has(room.room_type)) return { ...room, operators: [] };
        const crew = presetForShift(presetBySlot.get(room.slot_id) ?? [], shiftIndex);
        for (const id of crew) working.add(id);
        return { ...room, operators: crew };
    });

    const seated = new Set(api.boardRooms.flatMap((room) => (seatsOf(room, catalog) > 0 ? room.operators : [])));
    const offDuty = [...seated].filter((id) => !working.has(id)).length;

    return { rooms, offDuty };
}
