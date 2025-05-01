import type { GachaTable } from "../../../../../../../types/impl/lib/impl/local/impl/gamedata/impl/gacha";
import { STATIC_DATA } from "../../../handler";

export const getAll = (): GachaTable => {
    const data = STATIC_DATA?.GACHA_TABLE as GachaTable;
    return data;
};
