import { getAll } from "..";
import type { Module } from "../../../../../../../../types/impl/lib/impl/local/impl/gamedata/impl/modules";

export const get = async (id: string): Promise<Module | null> => {
    const modules = (await getAll()).equipDict;
    return Object.values(modules).find((module) => module.uniEquipId === id) ?? null;
};