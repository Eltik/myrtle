import { ACESHIP_REPOSITORY } from "../..";
import type { Modules } from "../../../../../../../types/impl/lib/impl/local/impl/gamedata/impl/modules";
import { ExcelTables } from "../../../../../../../types/impl/lib/impl/local/impl/handler";
import { get as getModules } from "../../../handler/impl/get";
import { get } from "./impl/get";
import { getByCharId } from "./impl/getByCharacter";
import { getModuleDetails } from "./impl/getDetails";

export const getAll = async (): Promise<Modules> => {
    const data = (await getModules(ExcelTables.UNIEQUIP_TABLE)) as Modules;
    for (const module in data.equipDict) {
        Object.assign(data.equipDict[module], { id: module, image: `https://raw.githubusercontent.com/${ACESHIP_REPOSITORY}/main/equip/icon/${encodeURIComponent(data.equipDict[module].uniEquipIcon)}.png` });
    }
    return data;
};

export default {
    get,
    getByCharId,
    getModuleDetails,
};
