// Credit to https://github.com/WhoAteMyCQQkie/ArknightsDpsCompare/blob/main/damagecalc/damage_formulas.py
// But holy crap this is WAYY too complicated. I'm not even sure if I can implement this in the backend.
// Work on this later...

import type { CalculateDPSParams, CalculateNormalATKParams, CharAttr } from "~/types/impl/frontend/impl/dps-calculator";
import { calculateAttack } from "./impl/calculate-attack";
import { checkSpecs } from "./impl/check-specs";
import { getAttributes } from "./impl/get-attributes";
import { getBlackboard } from "./impl/get-blackboard";
import type { Skill } from "~/types/impl/api/static/skills";

export const AttributeKeys = ["atk", "attackSpeed", "baseAttackTime", "baseForceLevel", "blockCnt", "cost", "def", "hpRecoveryPerSec", "magicResistance", "massLevel", "maxDeckStackCnt", "maxDeployCount", "maxHp", "moveSpeed", "respawnTime", "spRecoveryPerSec", "tauntLevel"];

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

export const calculateDPS = (params: CalculateDPSParams) => {
    const operator = params.char;
    const skillData = params.char.skills.find((skill) => skill.skillId === params.operatorData.skillId);
    const enemy = params.enemy;
    const raidBuff = params.buffConfig;

    const raidBlackboard = {
        atk: raidBuff.atkpct / 100,
        atk_override: raidBuff.atk,
        attack_speed: raidBuff.ats,
        sp_recovery_per_sec: raidBuff.cdr / 100,
        base_atk: raidBuff.base_atk / 100,
        damage_scale: 1 + raidBuff.damage_scale / 100,
    };

    const levelData = skillData?.static?.levels[params.operatorData.skillIndex];
    const blackboard = getBlackboard(levelData?.blackboard ?? []);

    console.log(`Calculating DPS for ${operator.name} with skill ${skillData?.skillId} at skill level ${params.operatorData.skillIndex} against an enemy with ${enemy.def} defense and ${enemy.res} resistance...`);

    const attr = getAttributes(
        operator,
        {
            equipId: params.operatorData.equipId,
            equipLevel: params.operatorData.equipLevel,
            favor: params.operatorData.favor,
            level: params.operatorData.level,
            potentialRank: params.operatorData.potentialRank,
            skillId: params.operatorData.skillId,
        },
        params.operatorData.phaseIndex,
    );

    Object.assign(blackboard, {
        id: skillData?.skillId,
    });

    Object.assign(attr.buffList, {
        skill: blackboard,
    });

    Object.assign(attr, {
        skillId: blackboard.id,
    });

    if (params.options.token && (checkSpecs(params.char.id ?? "", "token") || checkSpecs(params.operatorData.skillId, "token"))) {
        //var tokenId = checkSpecs(params.char.id ?? "", "token") || checkSpecs(params.operatorData.skillId, "token");
        //getTokenAtkHp(attr, tokenId, log);
    }

    if (raidBlackboard.base_atk !== 0) {
        const delta = (attr.basic.atk ?? 1) * raidBlackboard.base_atk;
        Object.assign(attr.basic, {
            atk: Math.round((attr.basic.atk ?? 1) + delta),
        });
    }

    const _backup = {
        basic: { ...attr.basic },
    };

    let normalAttack = null;
    let skillAttack = null;

    if (!checkSpecs(params.operatorData.skillId, "overdrive")) {
        skillAttack = calculateAttack(attr as unknown as CharAttr, params.options, params.operatorData, enemy, raidBlackboard, true, params.char, levelData as unknown as Skill["levels"][0]);

        attr.basic = _backup.basic;

        normalAttack = calculateAttack(attr as unknown as CharAttr, params.options, params.operatorData, enemy, raidBlackboard, false, params.char, levelData as unknown as Skill["levels"][0]);
    } else {
        const od_p1 = calculateAttack(attr as unknown as CharAttr, params.options, params.operatorData, enemy, raidBlackboard, true, params.char, levelData as unknown as Skill["levels"][0]);

        attr.basic = Object.assign({}, _backup.basic);
        Object.assign(attr.char, {
            options: {
                overdrive_mode: true,
            },
        });
        const od_p2 = calculateAttack(attr as unknown as CharAttr, params.options, params.operatorData, enemy, raidBlackboard, true, params.char, levelData as unknown as Skill["levels"][0]);

        // merge result
        const merged = Object.assign({}, od_p2);
        merged.dur = Object.assign({}, od_p2.dur);
        ["totalDamage", "totalHeal", "extraDamage", "extraHeal"].forEach((key) => {
            Object.assign(merged, {
                [key]: typeof merged[key as keyof typeof merged] === "number" && typeof od_p1[key as keyof typeof od_p1] === "number" ? (merged[key as keyof typeof merged] as number) + (od_p1[key as keyof typeof od_p1] as number) : merged[key as keyof typeof merged],
            });
        });
        for (let i = 0; i < merged.damagePool.length; ++i) {
            merged.damagePool[i]! += od_p1.damagePool[i] ?? 0;
            merged.extraDamagePool[i]! += od_p1.extraDamagePool[i] ?? 0;
        }
        ["attackCount", "hitCount", "duration", "stunDuration", "prepDuration"].forEach((key) => {
            Object.assign(merged.dur, {
                [key]: typeof merged.dur[key as keyof typeof merged.dur] === "number" && typeof od_p1.dur[key as keyof typeof od_p1.dur] === "number" ? (merged.dur[key as keyof typeof merged.dur] as number) + (od_p1.dur[key as keyof typeof od_p1.dur] as number) : merged.dur[key as keyof typeof merged.dur],
            });
        });
        const tm = merged.dur.duration + merged.dur.stunDuration + merged.dur.prepDuration;
        merged.dps = merged.totalDamage / tm;
        merged.hps = merged.totalHeal / tm;
        skillAttack = merged;

        attr.basic = Object.assign({}, _backup.basic);
        Object.assign(attr.char, {
            options: {
                overdrive_mode: false,
            },
        });
        normalAttack = calculateAttack(attr as unknown as CharAttr, params.options, params.operatorData, enemy, raidBlackboard, false, params.char, levelData as unknown as Skill["levels"][0]);
    }

    const globalDps = Math.round((normalAttack.totalDamage + skillAttack.totalDamage) / (normalAttack.dur.duration + normalAttack.dur.stunDuration + skillAttack.dur.duration + skillAttack.dur.prepDuration));
    const globalHps = Math.round((normalAttack.totalHeal + skillAttack.totalHeal) / (normalAttack.dur.duration + normalAttack.dur.stunDuration + skillAttack.dur.duration + skillAttack.dur.prepDuration));
    const killTime = 0;
    return {
        normal: normalAttack,
        skill: skillAttack,
        skillName: levelData?.name,

        killTime: killTime,
        globalDps,
        globalHps,
    };
};
