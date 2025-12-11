import { isEqual, range } from "lodash-es";
import type { AttributeData, Operator } from "~/types/api";

/**
 * Get operator attribute stats based on level, phase, trust, potential, and module
 */
export function getOperatorAttributeStats(
    operator: Operator,
    metadata: {
        phaseIndex: number;
        favorPoint: number;
        potentialRank: number;
        moduleId: string;
        moduleLevel: number;
    },
    level: number,
): AttributeData | null {
    const phase = operator.phases[metadata.phaseIndex];

    const keyFrames = phase?.AttributesKeyFrames;
    if (!keyFrames || keyFrames.length === 0) {
        return null;
    }

    if (level < 1 || level > phase.MaxLevel) {
        return null;
    }

    const trust = metadata.favorPoint;
    const potential = metadata.potentialRank;

    const maxLevel = phase.MaxLevel;

    const startingKeyFrame = keyFrames[0];
    const finalKeyFrame = keyFrames[keyFrames.length - 1];

    if (!startingKeyFrame || !finalKeyFrame) {
        return null;
    }

    const { MaxHp: maxHp, Atk: atk, Def: def, MagicResistance: res, Cost: dp, BlockCnt: blockCnt, RespawnTime: redeploy, BaseAttackTime: baseAttackTime } = startingKeyFrame.Data;

    const { MaxHp: finalMaxHp, Atk: finalMaxAtk, Def: finalMaxDef, MagicResistance: finalMaxRes } = finalKeyFrame.Data;

    // Trust bonuses
    const trustBonuses = doStatsChange(operator, metadata.phaseIndex) ? getStatIncreaseAtTrust(operator, trust) : { maxHp: 0, atk: 0, def: 0, magicResistance: 0 };

    // Potential bonuses
    const potBonuses = doStatsChange(operator, metadata.phaseIndex)
        ? getStatIncreaseAtPotential(operator, potential)
        : {
              health: 0,
              attackPower: 0,
              defense: 0,
              artsResistance: 0,
              attackSpeed: 0,
              dpCost: 0,
              redeployTimeInSeconds: 0,
          };

    // Calculate final stats
    const health = linearInterpolateByLevel(level, maxLevel, maxHp, finalMaxHp) + trustBonuses.maxHp + potBonuses.health;
    const attackPower = linearInterpolateByLevel(level, maxLevel, atk, finalMaxAtk) + trustBonuses.atk + potBonuses.attackPower;
    const defense = linearInterpolateByLevel(level, maxLevel, def, finalMaxDef) + trustBonuses.def + potBonuses.defense;
    const artsResistance = linearInterpolateByLevel(level, maxLevel, res, finalMaxRes) + trustBonuses.magicResistance + potBonuses.artsResistance;

    const redeployTimeInSeconds = redeploy + potBonuses.redeployTimeInSeconds;
    const dpCost = dp + potBonuses.dpCost;
    const blockCount = blockCnt;

    // ASPD calculation
    const secondsPerAttack = calculateSecondsPerAttack(baseAttackTime, 100 + potBonuses.attackSpeed);

    const stats: AttributeData = {
        Atk: attackPower,
        AttackSpeed: secondsPerAttack,
        BaseAttackTime: baseAttackTime,
        BaseForceLevel: finalKeyFrame.Data.BaseForceLevel,
        BlockCnt: blockCount,
        Cost: dpCost,
        Def: defense,
        DisarmedCombatImmune: finalKeyFrame.Data.DisarmedCombatImmune,
        FrozenImmune: finalKeyFrame.Data.FrozenImmune,
        HpRecoveryPerSec: finalKeyFrame.Data.HpRecoveryPerSec,
        LevitateImmune: finalKeyFrame.Data.LevitateImmune,
        MagicResistance: artsResistance,
        MassLevel: finalKeyFrame.Data.MassLevel,
        MaxDeckStackCnt: finalKeyFrame.Data.MaxDeckStackCnt,
        MaxDeployCount: finalKeyFrame.Data.MaxDeployCount,
        MaxHp: health,
        MoveSpeed: finalKeyFrame.Data.MoveSpeed,
        RespawnTime: redeployTimeInSeconds,
        SilenceImmune: finalKeyFrame.Data.SilenceImmune,
        SleepImmune: finalKeyFrame.Data.SleepImmune,
        SpRecoveryPerSec: finalKeyFrame.Data.SpRecoveryPerSec,
        StunImmune: finalKeyFrame.Data.StunImmune,
        TauntLevel: finalKeyFrame.Data.TauntLevel,
    };

    return stats;
}

function doStatsChange(operator: Operator, phaseIndex: number): boolean {
    const phase = operator.phases[phaseIndex];
    if (!phase) return false;

    const startingKeyFrame = phase.AttributesKeyFrames[0];
    const finalKeyFrame = phase.AttributesKeyFrames[phase.AttributesKeyFrames.length - 1];

    return !isEqual(startingKeyFrame?.Data, finalKeyFrame?.Data);
}

function getStatIncreaseAtTrust(operator: Operator, rawTrust: number): { maxHp: number; atk: number; def: number; magicResistance: number } {
    if (!operator.favorKeyFrames || operator.favorKeyFrames.length === 0) {
        return { maxHp: 0, atk: 0, def: 0, magicResistance: 0 };
    }

    const trust = Math.min(100, rawTrust);
    const maxTrust = operator.favorKeyFrames[operator.favorKeyFrames.length - 1]?.Data;

    return {
        maxHp: Math.round((trust * (maxTrust?.MaxHp ?? 0)) / 100),
        atk: Math.round((trust * (maxTrust?.Atk ?? 0)) / 100),
        def: Math.round((trust * (maxTrust?.Def ?? 0)) / 100),
        magicResistance: Math.round((trust * (maxTrust?.MagicResistance ?? 0)) / 100),
    };
}

function getStatIncreaseAtPotential(
    operator: Operator,
    potential: number,
): {
    health: number;
    attackPower: number;
    defense: number;
    artsResistance: number;
    dpCost: number;
    attackSpeed: number;
    redeployTimeInSeconds: number;
} {
    const initialIncreases = {
        health: 0,
        attackPower: 0,
        defense: 0,
        artsResistance: 0,
        dpCost: 0,
        attackSpeed: 0,
        redeployTimeInSeconds: 0,
    };

    if (potential === 0) {
        return initialIncreases;
    }

    const relevantStatIncreases = range(1, potential + 1).map((p) => getPotentialStatIncrease(operator, p));

    return relevantStatIncreases.reduce((vals, previous) => {
        return {
            health: vals.health + previous.health,
            attackPower: vals.attackPower + previous.attackPower,
            defense: vals.defense + previous.defense,
            artsResistance: vals.artsResistance + previous.artsResistance,
            dpCost: vals.dpCost + previous.dpCost,
            attackSpeed: vals.attackSpeed + previous.attackSpeed,
            redeployTimeInSeconds: vals.redeployTimeInSeconds + previous.redeployTimeInSeconds,
        };
    }, initialIncreases);
}

function getPotentialStatIncrease(
    operator: Operator,
    potential: number,
): {
    health: number;
    attackPower: number;
    defense: number;
    artsResistance: number;
    dpCost: number;
    attackSpeed: number;
    redeployTimeInSeconds: number;
} {
    const statChanges = {
        health: 0,
        attackPower: 0,
        defense: 0,
        artsResistance: 0,
        dpCost: 0,
        attackSpeed: 0,
        redeployTimeInSeconds: 0,
    };

    if (potential === 0) {
        return statChanges;
    }

    const pot = operator.potentialRanks[potential - 1];

    if (!pot?.Buff?.Attributes?.AttributeModifiers) {
        return statChanges;
    }

    const attribType = pot.Buff.Attributes.AttributeModifiers[0]?.AttributeType;
    const attribChange = pot.Buff.Attributes.AttributeModifiers[0]?.Value ?? 0;

    switch (attribType) {
        case "MAX_HP":
            statChanges.health += attribChange;
            break;
        case "ATK":
            statChanges.attackPower += attribChange;
            break;
        case "DEF":
            statChanges.defense += attribChange;
            break;
        case "MAGIC_RESISTANCE":
            statChanges.artsResistance += attribChange;
            break;
        case "COST":
            statChanges.dpCost += attribChange;
            break;
        case "ATTACK_SPEED":
            statChanges.attackSpeed += attribChange;
            break;
        case "RESPAWN_TIME":
            statChanges.redeployTimeInSeconds += attribChange;
            break;
    }

    return statChanges;
}

function linearInterpolateByLevel(level: number, maxLevel: number, baseValue: number, maxValue: number): number {
    return Math.round(baseValue + ((level - 1) * (maxValue - baseValue)) / (maxLevel - 1));
}

function calculateSecondsPerAttack(baseAttackTime: number, aspd: number): number {
    return Math.round((baseAttackTime * 30) / (aspd / 100.0)) / 30;
}
