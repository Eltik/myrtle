import type { CharPatchTable } from "../../../../../../../types/impl/lib/impl/local/impl/gamedata/impl/charpatch";
import { STATIC_DATA } from "../../../handler";

export const getAll = (): CharPatchTable => {
    const data = STATIC_DATA?.CHAR_PATCH_TABLE as CharPatchTable;
    return data;
};
