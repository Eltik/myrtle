import type { Handbook, HandbookItem } from "../../../../../../../types/impl/lib/impl/local/impl/gamedata/impl/handbook";
import { STATIC_DATA } from "../../../handler";

export const getAll = (): Handbook => {
    const data = STATIC_DATA?.HANDBOOK_INFO_TABLE as Handbook;
    return data;
};

export default (id: string): HandbookItem | null => {
    const handbookItems = getAll().handbookDict;
    return Object.values(handbookItems).find((item) => item.charID === id) ?? null;
};
