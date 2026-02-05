import { getRarityStarCount } from "~/lib/utils";
import type { CharacterData, CharacterStatic, UserCharacterModule, UserFavorKeyFrame, UserPotentialRank } from "~/types/api/impl/user";

// Max level by rarity for each elite phase [E0, E1, E2]
const MAX_LEVEL_BY_RARITY: Record<number, number[]> = {
    6: [50, 80, 90],
    5: [50, 70, 80],
    4: [45, 60, 70],
    3: [40, 55, 55],
    2: [30, 30, 30],
    1: [30, 30, 30],
};

// Max elite phase by rarity
const MAX_ELITE_BY_RARITY: Record<number, number> = {
    6: 2,
    5: 2,
    4: 2,
    3: 1,
    2: 0,
    1: 0,
};

/**
 * Check if an operator is fully maxed (for glow effect).
 * Maxed means: max potential (6), max elite phase, max level, all skills M3 (or skill 7 for 3-stars),
 * and all unlocked modules at level 3 for E2 operators.
 */
export function checkIsMaxed(data: CharacterData): boolean {
    const operator = data.static as CharacterStatic | null;
    const operatorRarity = operator?.rarity ?? "TIER_1";
    const starCount = getRarityStarCount(operatorRarity);

    const maxElite = MAX_ELITE_BY_RARITY[starCount] ?? 2;
    const maxLevel = MAX_LEVEL_BY_RARITY[starCount]?.[maxElite] ?? 90;

    if (data.evolvePhase !== maxElite) return false;
    if (data.level !== maxLevel) return false;
    if (starCount > 2 && data.mainSkillLvl !== 7) return false;

    if (maxElite === 2) {
        if (!data.skills.every((skill) => skill.specializeLevel === 3)) return false;

        const unlockedModules =
            operator?.modules?.filter((mod) => {
                const equipData = data.equip[mod.uniEquipId];
                return mod.typeName1 !== "ORIGINAL" && equipData && equipData.level > 0 && equipData.locked !== 1;
            }) ?? [];
        if (unlockedModules.length > 0 && !unlockedModules.every((mod) => data.equip[mod.uniEquipId]?.level === 3)) return false;
    }

    if (data.potentialRank !== 5) return false;

    return true;
}

export function linearInterpolateByLevel(level: number, maxLevel: number, baseValue: number, maxValue: number): number {
    if (maxLevel === 1) return baseValue;
    return Math.round(baseValue + ((level - 1) * (maxValue - baseValue)) / (maxLevel - 1));
}

export function getStatIncreaseAtTrust(favorKeyFrames: UserFavorKeyFrame[] | undefined, rawTrust: number): { maxHp: number; atk: number; def: number } {
    if (!favorKeyFrames || favorKeyFrames.length === 0) {
        return { maxHp: 0, atk: 0, def: 0 };
    }

    // Trust caps at 100 for stats even though favor points go to 200
    const trust = Math.min(100, Math.floor(rawTrust / 2));
    const maxTrustFrame = favorKeyFrames[favorKeyFrames.length - 1]?.Data;

    return {
        maxHp: Math.round((trust * (maxTrustFrame?.MaxHp ?? 0)) / 100),
        atk: Math.round((trust * (maxTrustFrame?.Atk ?? 0)) / 100),
        def: Math.round((trust * (maxTrustFrame?.Def ?? 0)) / 100),
    };
}

export function getStatIncreaseAtPotential(
    potentialRanks: UserPotentialRank[] | undefined,
    potential: number,
): {
    health: number;
    attackPower: number;
    defense: number;
    artsResistance: number;
    dpCost: number;
    attackSpeed: number;
    blockCnt: number;
} {
    const statChanges = {
        health: 0,
        attackPower: 0,
        defense: 0,
        artsResistance: 0,
        dpCost: 0,
        attackSpeed: 0,
        blockCnt: 0,
    };

    if (!potentialRanks || potential === 0) {
        return statChanges;
    }

    for (let p = 1; p <= potential; p++) {
        const pot = potentialRanks[p - 1];
        if (!pot) continue;

        // biome-ignore lint/suspicious/noExplicitAny: API may return camelCase but types are PascalCase
        const potAny = pot as any;

        const buff = potAny.buff ?? pot.Buff;
        if (!buff) continue;

        const attributes = buff.attributes ?? buff.Attributes;
        if (!attributes) continue;

        const modifiers = attributes.attributeModifiers ?? attributes.AttributeModifiers;
        if (!modifiers || modifiers.length === 0) continue;

        const modifier = modifiers[0];
        if (!modifier) continue;

        const attribType = modifier.attributeType ?? modifier.AttributeType;
        const attribChange = modifier.value ?? modifier.Value ?? 0;

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
                break;
            case "BLOCK_CNT":
                statChanges.blockCnt += attribChange;
                break;
        }
    }

    return statChanges;
}

export function getModuleStatIncrease(
    modules: UserCharacterModule[] | undefined,
    currentEquip: string | null,
    equipData: Record<string, { hide: number; locked: number; level: number }>,
): {
    maxHp: number;
    atk: number;
    def: number;
    magicResistance: number;
    cost: number;
    attackSpeed: number;
    blockCnt: number;
} {
    const statChanges = {
        maxHp: 0,
        atk: 0,
        def: 0,
        magicResistance: 0,
        cost: 0,
        attackSpeed: 0,
        blockCnt: 0,
    };

    if (!modules || !currentEquip) {
        return statChanges;
    }

    const equippedModule = modules.find((m) => m.uniEquipId === currentEquip);
    if (!equippedModule?.data?.phases) {
        return statChanges;
    }

    const moduleLevel = equipData[currentEquip]?.level ?? 0;
    if (moduleLevel <= 0) {
        return statChanges;
    }

    const modulePhase = equippedModule.data.phases[moduleLevel - 1];
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
            case "magic_resistance":
                statChanges.magicResistance += attr.value;
                break;
            case "cost":
                statChanges.cost += attr.value;
                break;
            case "attack_speed":
                statChanges.attackSpeed += attr.value;
                break;
            case "block_cnt":
                statChanges.blockCnt += attr.value;
                break;
        }
    }

    return statChanges;
}

export function getAttributeStats(data: CharacterData, operator: CharacterStatic | null) {
    const phase = operator?.phases?.[data.evolvePhase];
    const keyFrames = phase?.AttributesKeyFrames;

    if (!keyFrames || keyFrames.length === 0) return null;

    const firstFrame = keyFrames[0];
    const lastFrame = keyFrames[keyFrames.length - 1];

    if (!firstFrame || !lastFrame) return null;

    const maxLevel = phase.MaxLevel;

    const baseMaxHp = firstFrame.Data.MaxHp;
    const baseAtk = firstFrame.Data.Atk;
    const baseDef = firstFrame.Data.Def;
    const baseRes = firstFrame.Data.MagicResistance;
    const baseCost = firstFrame.Data.Cost;
    const baseBlockCnt = firstFrame.Data.BlockCnt;

    const finalMaxHp = lastFrame.Data.MaxHp;
    const finalAtk = lastFrame.Data.Atk;
    const finalDef = lastFrame.Data.Def;
    const finalRes = lastFrame.Data.MagicResistance;

    const trustBonuses = getStatIncreaseAtTrust(operator?.favorKeyFrames, data.favorPoint);
    const potBonuses = getStatIncreaseAtPotential(operator?.potentialRanks, data.potentialRank);
    const modBonuses = data.evolvePhase === 2 ? getModuleStatIncrease(operator?.modules, data.currentEquip, data.equip) : { maxHp: 0, atk: 0, def: 0, magicResistance: 0, cost: 0, attackSpeed: 0, blockCnt: 0 };

    const maxHp = linearInterpolateByLevel(data.level, maxLevel, baseMaxHp, finalMaxHp) + trustBonuses.maxHp + potBonuses.health + modBonuses.maxHp;
    const atk = linearInterpolateByLevel(data.level, maxLevel, baseAtk, finalAtk) + trustBonuses.atk + potBonuses.attackPower + modBonuses.atk;
    const def = linearInterpolateByLevel(data.level, maxLevel, baseDef, finalDef) + trustBonuses.def + potBonuses.defense + modBonuses.def;
    const magicResistance = linearInterpolateByLevel(data.level, maxLevel, baseRes, finalRes) + potBonuses.artsResistance + modBonuses.magicResistance;
    const cost = baseCost + potBonuses.dpCost + modBonuses.cost;
    const blockCnt = baseBlockCnt + potBonuses.blockCnt + modBonuses.blockCnt;

    return {
        maxHp,
        atk,
        def,
        magicResistance,
        cost,
        blockCnt,
    };
}
