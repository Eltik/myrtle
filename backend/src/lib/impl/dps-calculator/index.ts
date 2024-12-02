// Credit to https://github.com/WhoAteMyCQQkie/ArknightsDpsCompare/blob/main/damagecalc/damage_formulas.py
// But holy crap this is WAYY too complicated. I'm not even sure if I can implement this in the backend.
// Work on this later...

import type { CalculatorParams } from "../../../types/impl/lib/impl/dps-calculator";

export const calculateNormalAttack = (params: CalculatorParams): number => {
    const finalAtk = params.operatorPhase.data.atk * (1 + (params.extraBuffs?.[0] ?? 0) + params.buffs.attack) + (params.extraBuffs?.[1] ?? 0) + params.buffs.attackFlat;

    let hitDmg: number;
    if (!params.isPhysical) {
        hitDmg = Math.max(finalAtk * (1 - params.enemy.res / 100), finalAtk * 0.05);
    } else {
        hitDmg = Math.max(finalAtk - params.enemy.defense, finalAtk * 0.05);
    }

    const dps = (((((params.hits ?? 1) * hitDmg) / params.operatorPhase.data.baseAttackTime) * (params.operatorPhase.data.attackSpeed + (params.extraBuffs?.[2] ?? 0))) / 100) * (params.aoe ?? 1);
    return dps;
};
