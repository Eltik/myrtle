import { getAll } from "..";
import type { Module } from "../../../../../../../../types/impl/lib/impl/local/impl/gamedata/impl/modules";

export const getByCharId = async (id: string): Promise<Module[]> => {
    const modules = (await getAll()).equipDict;
    return Object.values(modules).filter((module) => module.charId === id);
};
