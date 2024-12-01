import type { Range, Ranges } from "../../../../../../../types/impl/lib/impl/local/impl/gamedata/impl/ranges";
import { STATIC_DATA } from "../../../handler";

export const getAll = (): Ranges => {
    const data = STATIC_DATA?.RANGE_TABLE as Ranges;
    return data;
};

export default (id: string): Range | null => {
    const ranges = getAll();
    for (const range of Object.values(ranges)) {
        if (range.id === id) return range;
    }

    return null;
};
