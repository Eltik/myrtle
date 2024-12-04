// Credit to https://github.com/WhoAteMyCQQkie/ArknightsDpsCompare/blob/main/damagecalc/damage_formulas.py
// But holy crap this is WAYY too complicated. I'm not even sure if I can implement this in the backend.
// Work on this later...

import { init as initOperators } from "./impl/operators";

export const AttributeKeys = ["atk", "attackSpeed", "baseAttackTime", "baseForceLevel", "blockCnt", "cost", "def", "hpRecoveryPerSec", "magicResistance", "massLevel", "maxDeckStackCnt", "maxDeployCount", "maxHp", "moveSpeed", "respawnTime", "spRecoveryPerSec", "tauntLevel"];

export const init = async () => {
    await initOperators();
};
