import type { CalculateNormalATKParams } from "../../../../types/impl/lib/impl/dps-calculator";

export const calculateNormalAttack = (params: CalculateNormalATKParams): number => {
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
