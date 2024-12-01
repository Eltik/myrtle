import type { Item, Materials } from "../../../../../../../types/impl/lib/impl/local/impl/gamedata/impl/materials";
import { STATIC_DATA } from "../../../handler";

export const getAll = (): Materials => {
    const data = STATIC_DATA?.ITEM_TABLE as Materials;
    return data;
};

export default (id: string): Item | null => {
    const materials = getAll().items;
    return Object.values(materials).find((material) => material.itemId === id) ?? null;
};
