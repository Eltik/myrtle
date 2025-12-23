import type { CharacterData, CharacterStatic, UserCharacterModule, UserFavorKeyFrame, UserPotentialRank } from "~/types/api/impl/user";

// Linear interpolation helper
export function linearInterpolateByLevel(level: number, maxLevel: number, baseValue: number, maxValue: number): number {
    if (maxLevel === 1) return baseValue;
    return Math.round(baseValue + ((level - 1) * (maxValue - baseValue)) / (maxLevel - 1));
}

// Get trust bonus stats (trust only affects HP, ATK, and DEF - not magic resistance)
export function getStatIncreaseAtTrust(favorKeyFrames: UserFavorKeyFrame[] | undefined, rawTrust: number): { maxHp: number; atk: number; def: number } {
    if (!favorKeyFrames || favorKeyFrames.length === 0) {
        return { maxHp: 0, atk: 0, def: 0 };
    }

    // Trust is capped at 100 for stat calculations (even though favor points go to 200)
    const trust = Math.min(100, Math.floor(rawTrust / 2));
    const maxTrustFrame = favorKeyFrames[favorKeyFrames.length - 1]?.Data;

    return {
        maxHp: Math.round((trust * (maxTrustFrame?.MaxHp ?? 0)) / 100),
        atk: Math.round((trust * (maxTrustFrame?.Atk ?? 0)) / 100),
        def: Math.round((trust * (maxTrustFrame?.Def ?? 0)) / 100),
    };
}

// Get potential bonus stats
// Handles both camelCase (actual API response) and PascalCase (TypeScript types) property access
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

    // Aggregate bonuses from potential ranks 1 through current potential
    for (let p = 1; p <= potential; p++) {
        const pot = potentialRanks[p - 1];
        if (!pot) continue;

        // Access using both camelCase (actual API) and PascalCase (TypeScript types)
        // biome-ignore lint/suspicious/noExplicitAny: API may return camelCase but types are PascalCase
        const potAny = pot as any;

        // Try camelCase first (actual API), then PascalCase (TypeScript types)
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

        // AttributeType is a string like "MAX_HP", "ATK", "DEF", etc.
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
                // Redeploy time reduction (e.g., -4 seconds)
                break;
            case "BLOCK_CNT":
                statChanges.blockCnt += attribChange;
                break;
        }
    }

    return statChanges;
}

// Get module bonus stats
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

    // Module phases are 0-indexed, level 1 = phase[0], level 2 = phase[1], etc.
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

// Calculate operator stats based on level, phase, trust, potential, and modules
export function getAttributeStats(data: CharacterData, operator: CharacterStatic | null) {
    const phase = operator?.phases?.[data.evolvePhase];
    const keyFrames = phase?.AttributesKeyFrames;

    if (!keyFrames || keyFrames.length === 0) return null;

    const firstFrame = keyFrames[0];
    const lastFrame = keyFrames[keyFrames.length - 1];

    if (!firstFrame || !lastFrame) return null;

    const maxLevel = phase.MaxLevel;

    // Base stats from keyframes
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

    // Get trust bonuses
    const trustBonuses = getStatIncreaseAtTrust(operator?.favorKeyFrames, data.favorPoint);

    // Get potential bonuses
    const potBonuses = getStatIncreaseAtPotential(operator?.potentialRanks, data.potentialRank);

    // Get module bonuses (only at E2)
    const modBonuses = data.evolvePhase === 2 ? getModuleStatIncrease(operator?.modules, data.currentEquip, data.equip) : { maxHp: 0, atk: 0, def: 0, magicResistance: 0, cost: 0, attackSpeed: 0, blockCnt: 0 };

    // Calculate final stats with all bonuses
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
