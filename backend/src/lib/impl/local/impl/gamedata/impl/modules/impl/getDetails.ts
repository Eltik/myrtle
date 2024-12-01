import type { BattleEquip, ModuleData } from "../../../../../../../../types/impl/lib/impl/local/impl/gamedata/impl/modules";
import { STATIC_DATA } from "../../../../handler";

export const getModuleDetails = (id: string): ModuleData | null => {
    const modules = getBattleEquip();
    return modules[id] ?? null;
};

const getBattleEquip = (): BattleEquip => {
    const data = STATIC_DATA?.BATTLE_EQUIP_TABLE as BattleEquip;
    return data;
};
