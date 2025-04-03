import { FEXLI_REPOSITORY } from "../..";
import type { Modules } from "../../../../../../../types/impl/lib/impl/local/impl/gamedata/impl/modules";
import { STATIC_DATA } from "../../../handler";
import { get } from "./impl/get";
import { getByCharId } from "./impl/getByCharacter";
import { getModuleDetails } from "./impl/getDetails";

export const getAll = (): Modules => {
    const data = STATIC_DATA?.UNIEQUIP_TABLE as Modules;
    for (const module in data.equipDict) {
        Object.assign(data.equipDict[module], { id: module, image: `https://raw.githubusercontent.com/${FEXLI_REPOSITORY}/main/equip/${encodeURIComponent(data.equipDict[module].uniEquipIcon)}.png` });
    }
    return data;
};

export default {
    get,
    getByCharId,
    getModuleDetails,
};
