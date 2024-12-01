import { getAll } from "..";
import type { Module } from "../../../../../../../../types/impl/lib/impl/local/impl/gamedata/impl/modules";

export const getByCharId = (id: string): Module[] => {
    const modules = getAll().equipDict;
    return Object.values(modules).filter((module) => module.charId === id);
};
