import { getAll } from "..";
import type { Module } from "../../../../../../../../types/impl/lib/impl/local/impl/gamedata/impl/modules";

export const get = (id: string): Module | null => {
    const modules = getAll().equipDict;
    return Object.values(modules).find((module) => module.uniEquipId === id) ?? null;
};
