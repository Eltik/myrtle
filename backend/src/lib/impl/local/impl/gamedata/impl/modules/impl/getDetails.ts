import type { BattleEquip, ModuleData } from "../../../../../../../../types/impl/lib/impl/local/impl/gamedata/impl/modules";
import { get as getModules } from "../../../../handler/impl/get";
import { ExcelTables } from "../../../../../../../../types/impl/lib/impl/local/impl/handler";

export const getModuleDetails = async (id: string): Promise<ModuleData | null> => {
    const modules = await getBattleEquip();
    return modules[id] ?? null;
};

const getBattleEquip = async (): Promise<BattleEquip> => {
    const data = (await getModules(ExcelTables.BATTLE_EQUIP_TABLE)) as BattleEquip;
    return data;
};
