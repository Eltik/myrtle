import { range } from "lodash-es";
import type { AttributeData, Module, Operator } from "~/types/api";

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

    // Trust bonuses (always apply)
    const trustBonuses = getStatIncreaseAtTrust(operator, trust);

    // Potential bonuses (always apply)
    const potBonuses = getStatIncreaseAtPotential(operator, potential);

    // Module bonuses (only at E2 phase and if module is selected)
    const selectedModule = metadata.moduleId ? operator.modules.find((m) => m.uniEquipId === metadata.moduleId) : null;
    const moduleBonuses = metadata.phaseIndex === 2 && selectedModule ? getModuleStatIncrease(selectedModule, metadata.moduleLevel) : { atk: 0, maxHp: 0, def: 0, attackSpeed: 0, magicResistance: 0, cost: 0, respawnTime: 0, blockCnt: 0 };

    // Calculate final stats
    const health = linearInterpolateByLevel(level, maxLevel, maxHp, finalMaxHp) + trustBonuses.maxHp + potBonuses.health + moduleBonuses.maxHp;
    const attackPower = linearInterpolateByLevel(level, maxLevel, atk, finalMaxAtk) + trustBonuses.atk + potBonuses.attackPower + moduleBonuses.atk;
    const defense = linearInterpolateByLevel(level, maxLevel, def, finalMaxDef) + trustBonuses.def + potBonuses.defense + moduleBonuses.def;
    const artsResistance = linearInterpolateByLevel(level, maxLevel, res, finalMaxRes) + trustBonuses.magicResistance + potBonuses.artsResistance + moduleBonuses.magicResistance;

    const redeployTimeInSeconds = redeploy + potBonuses.redeployTimeInSeconds + moduleBonuses.respawnTime;
    const dpCost = dp + potBonuses.dpCost + moduleBonuses.cost;
    const blockCount = blockCnt + moduleBonuses.blockCnt;

    // ASPD calculation (include module attack speed bonus)
    const totalAspdBonus = 100 + potBonuses.attackSpeed + moduleBonuses.attackSpeed;
    const secondsPerAttack = calculateSecondsPerAttack(baseAttackTime, totalAspdBonus);

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

/**
 * Get stat increase from module at a specific level
 */
function getModuleStatIncrease(
    module: Module,
    moduleLevel: number,
): {
    atk: number;
    maxHp: number;
    def: number;
    attackSpeed: number;
    magicResistance: number;
    cost: number;
    respawnTime: number;
    blockCnt: number;
} {
    const statChanges = {
        atk: 0,
        maxHp: 0,
        def: 0,
        attackSpeed: 0,
        magicResistance: 0,
        cost: 0,
        respawnTime: 0,
        blockCnt: 0,
    };

    if (!module.data?.phases || moduleLevel <= 0) {
        return statChanges;
    }

    const modulePhase = module.data.phases[moduleLevel - 1];
    if (!modulePhase?.attributeBlackboard) {
        return statChanges;
    }

    for (const attr of modulePhase.attributeBlackboard) {
        switch (attr.key) {
            case "atk":
                statChanges.atk += attr.value;
                break;
            case "max_hp":
                statChanges.maxHp += attr.value;
                break;
            case "def":
                statChanges.def += attr.value;
                break;
            case "attack_speed":
                statChanges.attackSpeed += attr.value;
                break;
            case "magic_resistance":
                statChanges.magicResistance += attr.value;
                break;
            case "cost":
                statChanges.cost += attr.value;
                break;
            case "respawn_time":
                statChanges.respawnTime += attr.value;
                break;
            case "block_cnt":
                statChanges.blockCnt += attr.value;
                break;
        }
    }

    return statChanges;
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

/**
 * Parse the numeric value from a potential description string
 * Examples:
 *   "Max HP +200" -> 200
 *   "DP Cost -1" -> -1
 *   "ATK +30" -> 30
 *   "Redeploy Time -4s" -> -4
 */
function parseValueFromDescription(description: string): number {
    // Match patterns like "+200", "-1", "+30", "-4s", etc.
    const match = description.match(/([+-]?\d+)/);
    if (match?.[1]) {
        return Number.parseInt(match[1], 10);
    }
    return 0;
}

/**
 * Get stat increase for a single potential rank
 * Uses camelCase property access (matching actual API response format)
 * Falls back to parsing description when value is missing (common in decoded FlatBuffers data)
 */
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
    if (!pot) {
        return statChanges;
    }

    // Access using camelCase (actual API response format) with fallback to PascalCase (TypeScript types)
    // biome-ignore lint/suspicious/noExplicitAny: API returns camelCase but types are PascalCase
    const potAny = pot as any;

    // Get description for fallback value parsing
    const description = potAny.description ?? pot.Description ?? "";

    // Try camelCase first (actual API), then PascalCase (TypeScript types)
    const buff = potAny.buff ?? pot.Buff;
    if (!buff) {
        return statChanges;
    }

    const attributes = buff.attributes ?? buff.Attributes;
    if (!attributes) {
        return statChanges;
    }

    const modifiers = attributes.attributeModifiers ?? attributes.AttributeModifiers;
    if (!modifiers || modifiers.length === 0) {
        return statChanges;
    }

    const modifier = modifiers[0];
    if (!modifier) {
        return statChanges;
    }

    const attribType = modifier.attributeType ?? modifier.AttributeType;
    // Try to get value from API, fallback to parsing from description if value is 0 or missing
    let attribChange = modifier.value ?? modifier.Value ?? 0;
    if (attribChange === 0 && description) {
        attribChange = parseValueFromDescription(description);
    }

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
