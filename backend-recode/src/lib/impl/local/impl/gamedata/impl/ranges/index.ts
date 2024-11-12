import type { Range, Ranges } from "../../../../../../../types/impl/lib/impl/local/impl/gamedata/impl/ranges";
import { ExcelTables } from "../../../../../../../types/impl/lib/impl/local/impl/handler";
import { get as getRanges } from "../../../handler/impl/get";

export const getAll = async (): Promise<Ranges> => {
    const data = (await getRanges(ExcelTables.RANGE_TABLE)) as Ranges;
    return data;
};

export default async (id: string): Promise<Range | null> => {
    const ranges = await getAll();
    for (const range of Object.values(ranges)) {
        if (range.id === id) return range;
    }

    return null;
};
