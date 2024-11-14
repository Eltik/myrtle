import type { Item, Materials } from "../../../../../../../types/impl/lib/impl/local/impl/gamedata/impl/materials";
import { ExcelTables } from "../../../../../../../types/impl/lib/impl/local/impl/handler";
import { get as getMaterials } from "../../../handler/impl/get";

export const getAll = async (): Promise<Materials> => {
    const data = (await getMaterials(ExcelTables.ITEM_TABLE)) as Materials;
    return data;
};

export default async (id: string): Promise<Item | null> => {
    const materials = (await getAll()).items;
    return Object.values(materials).find((material) => material.itemId === id) ?? null;
};
