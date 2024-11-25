import { range, isEqual } from "lodash-es";
import type { CharacterData } from "~/types/impl/api";
import { type BattleEquip } from "~/types/impl/api/static/modules";
import { type Operator } from "~/types/impl/api/static/operator";

/**
 * @author Huge thanks and credit to
 * https://github.com/SanityGoneAK/sanity-gone/blob/main/src/utils/character-stats.ts#L80
 * All credit goes to them for this function.
 */
export const getAttributeStats = (character: CharacterData, level: number, moduleData?: BattleEquip): Operator["phases"][number]["attributesKeyFrames"][number]["data"] | null => {
    const phase = getCurrentPhase(character);
    if (!phase) return null;

    const keyFrames = phase.attributesKeyFrames;
    if (!keyFrames || keyFrames.length === 0) {
        return null;
    }

    if (level < 1 || level > phase.maxLevel) {
        return null;
    }

    const trust = character.favorPoint;
    const potential = character.potentialRank;
    const moduleId = character.currentEquip;
    const moduleLevel = character.equip[moduleId as unknown as keyof typeof character.equip]?.level ?? 0;

    const maxLevel = phase.maxLevel;

    const startingKeyFrame = phase.attributesKeyFrames[0];
    const finalKeyFrame = phase.attributesKeyFrames[phase.attributesKeyFrames.length - 1];

    const { maxHp, atk, def, magicResistance: res, cost: dp, blockCnt, respawnTime: redeploy, baseAttackTime } = startingKeyFrame!.data;
    const { maxHp: finalMaxHp, atk: finalMaxAtk, def: finalMaxDef, magicResistance: finalMaxRes } = finalKeyFrame!.data;

    const {
        maxHp: trustHp,
        atk: trustAtk,
        def: trustDef,
        magicResistance: trustRes,
    } = doStatsChange(character)
        ? getStatIncreaseAtTrust(character.static!, trust)
        : {
              maxHp: 0,
              atk: 0,
              def: 0,
              magicResistance: 0,
          };

    const {
        health: potHealth,
        attackPower: potAttack,
        defense: potDefense,
        artsResistance: potRes,
        attackSpeed: potASPD,
        dpCost: potDp,
        redeployTimeInSeconds: potRedeploy,
    } = doStatsChange(character)
        ? getStatIncreaseAtPotential(character.static!, potential)
        : {
              health: 0,
              attackPower: 0,
              defense: 0,
              artsResistance: 0,
              attackSpeed: 0,
              dpCost: 0,
              redeployTimeInSeconds: 0,
          };

    const {
        atk: modAttack,
        max_hp: modHealth,
        def: modDefense,
        attack_speed: modASPD,
        magic_resistance: modRes,
        cost: modDp,
        respawn_time: modRedeploy,
        block_cnt: modBlock,
    } = moduleId && moduleData
        ? getModuleStatIncrease(moduleId, moduleLevel, moduleData)
        : {
              atk: 0,
              max_hp: 0,
              def: 0,
              attack_speed: 0,
              magic_resistance: 0,
              cost: 0,
              respawn_time: 0,
              block_cnt: 0,
          };

    const health = linearInterpolateByLevel(level, maxLevel, maxHp, finalMaxHp) + trustHp + potHealth + modHealth;
    const attackPower = linearInterpolateByLevel(level, maxLevel, atk, finalMaxAtk) + trustAtk + potAttack + modAttack;
    const defense = linearInterpolateByLevel(level, maxLevel, def, finalMaxDef) + trustDef + potDefense + modDefense;
    const artsResistance = linearInterpolateByLevel(level, maxLevel, res, finalMaxRes) + trustRes + potRes + modRes;

    const redeployTimeInSeconds = redeploy + potRedeploy + modRedeploy;
    const dpCost = dp + potDp + modDp;
    const blockCount = blockCnt + modBlock;

    // ASPD
    const secondsPerAttack = calculateSecondsPerAttack(baseAttackTime, 100 + potASPD + modASPD);

    const stats: Operator["phases"][number]["attributesKeyFrames"][number]["data"] = {
        atk: attackPower,
        attackSpeed: secondsPerAttack,
        baseAttackTime,
        baseForceLevel: finalKeyFrame!.data.baseForceLevel,
        blockCnt: blockCount,
        cost: dpCost,
        def: defense,
        disarmedCombatImmune: finalKeyFrame!.data.disarmedCombatImmune,
        frozenImmune: finalKeyFrame!.data.frozenImmune,
        hpRecoveryPerSec: finalKeyFrame!.data.hpRecoveryPerSec,
        levitateImmune: finalKeyFrame!.data.levitateImmune,
        magicResistance: artsResistance,
        massLevel: finalKeyFrame!.data.massLevel,
        maxDeckStackCnt: finalKeyFrame!.data.maxDeckStackCnt,
        maxDeployCount: finalKeyFrame!.data.maxDeployCount,
        maxHp: health,
        moveSpeed: finalKeyFrame!.data.moveSpeed,
        respawnTime: redeployTimeInSeconds,
        silenceImmune: finalKeyFrame!.data.silenceImmune,
        sleepImmune: finalKeyFrame!.data.sleepImmune,
        spRecoveryPerSec: finalKeyFrame!.data.spRecoveryPerSec,
        stunImmune: finalKeyFrame!.data.stunImmune,
        tauntLevel: finalKeyFrame!.data.tauntLevel,
    };

    return stats;
};

const doStatsChange = (data: CharacterData): boolean => {
    const phase = getCurrentPhase(data);
    if (!phase) return false;

    const startingKeyFrame = phase.attributesKeyFrames[0];
    const finalKeyFrame = phase.attributesKeyFrames[phase.attributesKeyFrames.length - 1];

    return !isEqual(startingKeyFrame?.data, finalKeyFrame?.data);
};

export const getOperatorAttributeStats = (
    character: Operator,
    metadata: {
        phaseIndex: number;
        favorPoint: number;
        potentialRank: number;
        moduleId: string;
        moduleLevel: number;
    },
    level: number,
    moduleData?: BattleEquip,
): Operator["phases"][number]["attributesKeyFrames"][number]["data"] | null => {
    const phase = character.phases[metadata.phaseIndex];

    const keyFrames = phase?.attributesKeyFrames;
    if (!keyFrames || keyFrames.length === 0) {
        return null;
    }

    if (level < 1 || level > phase.maxLevel) {
        return null;
    }

    const trust = metadata.favorPoint;
    const potential = metadata.potentialRank;
    const moduleId = metadata.moduleId;
    const moduleLevel = metadata.moduleLevel;

    const maxLevel = phase.maxLevel;

    const startingKeyFrame = phase.attributesKeyFrames[0];
    const finalKeyFrame = phase.attributesKeyFrames[phase.attributesKeyFrames.length - 1];

    const { maxHp, atk, def, magicResistance: res, cost: dp, blockCnt, respawnTime: redeploy, baseAttackTime } = startingKeyFrame!.data;
    const { maxHp: finalMaxHp, atk: finalMaxAtk, def: finalMaxDef, magicResistance: finalMaxRes } = finalKeyFrame!.data;

    const {
        maxHp: trustHp,
        atk: trustAtk,
        def: trustDef,
        magicResistance: trustRes,
    } = doOperatorStatsChange(character, metadata.phaseIndex)
        ? getStatIncreaseAtTrust(character, trust)
        : {
              maxHp: 0,
              atk: 0,
              def: 0,
              magicResistance: 0,
          };

    const {
        health: potHealth,
        attackPower: potAttack,
        defense: potDefense,
        artsResistance: potRes,
        attackSpeed: potASPD,
        dpCost: potDp,
        redeployTimeInSeconds: potRedeploy,
    } = doOperatorStatsChange(character, metadata.phaseIndex)
        ? getStatIncreaseAtPotential(character, potential)
        : {
              health: 0,
              attackPower: 0,
              defense: 0,
              artsResistance: 0,
              attackSpeed: 0,
              dpCost: 0,
              redeployTimeInSeconds: 0,
          };

    const {
        atk: modAttack,
        max_hp: modHealth,
        def: modDefense,
        attack_speed: modASPD,
        magic_resistance: modRes,
        cost: modDp,
        respawn_time: modRedeploy,
        block_cnt: modBlock,
    } = moduleId && moduleData
        ? getModuleStatIncrease(moduleId, moduleLevel, moduleData)
        : {
              atk: 0,
              max_hp: 0,
              def: 0,
              attack_speed: 0,
              magic_resistance: 0,
              cost: 0,
              respawn_time: 0,
              block_cnt: 0,
          };

    const health = linearInterpolateByLevel(level, maxLevel, maxHp, finalMaxHp) + trustHp + potHealth + modHealth;
    const attackPower = linearInterpolateByLevel(level, maxLevel, atk, finalMaxAtk) + trustAtk + potAttack + modAttack;
    const defense = linearInterpolateByLevel(level, maxLevel, def, finalMaxDef) + trustDef + potDefense + modDefense;
    const artsResistance = linearInterpolateByLevel(level, maxLevel, res, finalMaxRes) + trustRes + potRes + modRes;

    const redeployTimeInSeconds = redeploy + potRedeploy + modRedeploy;
    const dpCost = dp + potDp + modDp;
    const blockCount = blockCnt + modBlock;

    // ASPD
    const secondsPerAttack = calculateSecondsPerAttack(baseAttackTime, 100 + potASPD + modASPD);

    const stats: Operator["phases"][number]["attributesKeyFrames"][number]["data"] = {
        atk: attackPower,
        attackSpeed: secondsPerAttack,
        baseAttackTime,
        baseForceLevel: finalKeyFrame!.data.baseForceLevel,
        blockCnt: blockCount,
        cost: dpCost,
        def: defense,
        disarmedCombatImmune: finalKeyFrame!.data.disarmedCombatImmune,
        frozenImmune: finalKeyFrame!.data.frozenImmune,
        hpRecoveryPerSec: finalKeyFrame!.data.hpRecoveryPerSec,
        levitateImmune: finalKeyFrame!.data.levitateImmune,
        magicResistance: artsResistance,
        massLevel: finalKeyFrame!.data.massLevel,
        maxDeckStackCnt: finalKeyFrame!.data.maxDeckStackCnt,
        maxDeployCount: finalKeyFrame!.data.maxDeployCount,
        maxHp: health,
        moveSpeed: finalKeyFrame!.data.moveSpeed,
        respawnTime: redeployTimeInSeconds,
        silenceImmune: finalKeyFrame!.data.silenceImmune,
        sleepImmune: finalKeyFrame!.data.sleepImmune,
        spRecoveryPerSec: finalKeyFrame!.data.spRecoveryPerSec,
        stunImmune: finalKeyFrame!.data.stunImmune,
        tauntLevel: finalKeyFrame!.data.tauntLevel,
    };

    return stats;
};

const doOperatorStatsChange = (data: Operator, phaseIndex: number): boolean => {
    const phase = data.phases[phaseIndex];

    const startingKeyFrame = phase?.attributesKeyFrames[0];
    const finalKeyFrame = phase?.attributesKeyFrames[phase.attributesKeyFrames.length - 1];

    return !isEqual(startingKeyFrame?.data, finalKeyFrame?.data);
};

const getStatIncreaseAtTrust = (
    data: Operator,
    rawTrust: number,
): {
    maxHp: number;
    atk: number;
    def: number;
    magicResistance: number;
} => {
    if (data.favorKeyFrames == null) {
        throw new Error(`Can't get trust stat increase, favorKeyFrames is null.`);
    }

    const trust = Math.min(100, rawTrust);
    const maxTrust = data.favorKeyFrames[data.favorKeyFrames.length - 1]?.data;

    return {
        maxHp: Math.round((trust * (maxTrust?.maxHp ?? 0)) / 100),
        atk: Math.round((trust * (maxTrust?.atk ?? 0)) / 100),
        def: Math.round((trust * (maxTrust?.def ?? 0)) / 100),
        magicResistance: Math.round((trust * (maxTrust?.magicResistance ?? 0)) / 100),
    };
};

const getStatIncreaseAtPotential = (
    data: Operator,
    potential: number,
): {
    health: number;
    attackPower: number;
    defense: number;
    artsResistance: number;
    dpCost: number;
    attackSpeed: number;
    redeployTimeInSeconds: number;
    description: string | null;
} => {
    const initialIncreases = {
        health: 0,
        attackPower: 0,
        defense: 0,
        artsResistance: 0,
        dpCost: 0,
        attackSpeed: 0,
        redeployTimeInSeconds: 0,
        description: null,
    };
    if (potential === 0) {
        return initialIncreases;
    }

    const relevantStatIncreases = range(1, potential + 1).map((p) => getPotentialStatIncrease(data, p));
    return relevantStatIncreases.reduce((vals, previous) => {
        return {
            health: vals.health + previous.health,
            attackPower: vals.attackPower + previous.attackPower,
            defense: vals.defense + previous.defense,
            artsResistance: vals.artsResistance + previous.artsResistance,
            dpCost: vals.dpCost + previous.dpCost,
            attackSpeed: vals.attackSpeed + previous.attackSpeed,
            redeployTimeInSeconds: vals.redeployTimeInSeconds + previous.redeployTimeInSeconds,
            description: null,
        };
    }, initialIncreases);
};

const getModuleStatIncrease = (
    moduleId: string,
    moduleLevel: number,
    moduleData: BattleEquip,
): {
    atk: number;
    max_hp: number;
    def: number;
    attack_speed: number;
    magic_resistance: number;
    cost: number;
    respawn_time: number;
    block_cnt: number;
} => {
    const statChanges = {
        atk: 0,
        max_hp: 0,
        def: 0,
        attack_speed: 0,
        magic_resistance: 0,
        cost: 0,
        respawn_time: 0,
        block_cnt: 0,
    };

    const operatorModule = moduleData[moduleId];

    const modulePhase = operatorModule?.phases?.[moduleLevel - 1];

    const toCheck = modulePhase?.attributeBlackboard;

    if (!toCheck) {
        return statChanges;
    }

    toCheck.forEach((iv) => {
        if (!(iv.key in statChanges)) {
            throw new Error(`Unknown attribute modified: ${iv.key} with value of ${iv.value}`);
        }
        // @ts-expect-error - This is fine
        statChanges[iv.key] += iv.value;
    });

    return statChanges;
};

const linearInterpolateByLevel = (level: number, maxLevel: number, baseValue: number, maxValue: number): number => {
    return Math.round(baseValue + ((level - 1) * (maxValue - baseValue)) / (maxLevel - 1));
};

const calculateSecondsPerAttack = (baseAttackTime: number, aspd: number): number => {
    return Math.round((baseAttackTime * 30) / (aspd / 100.0)) / 30;
};

const getPotentialStatIncrease = (
    data: Operator,
    potential: number,
): {
    health: number;
    attackPower: number;
    defense: number;
    artsResistance: number;
    dpCost: number;
    attackSpeed: number;
    redeployTimeInSeconds: number;
    description: string | null;
} => {
    if (potential === 0) {
        return {
            artsResistance: 0,
            attackPower: 0,
            attackSpeed: 0,
            defense: 0,
            description: null,
            dpCost: 0,
            health: 0,
            redeployTimeInSeconds: 0,
        };
    }

    const potentialRanks = data.potentialRanks;
    const pot = potentialRanks[potential - 1];

    const statChanges = {
        health: 0,
        attackPower: 0,
        defense: 0,
        artsResistance: 0,
        dpCost: 0,
        attackSpeed: 0,
        redeployTimeInSeconds: 0,
        description: null,
    };

    if (pot?.buff == null) {
        let desc = pot?.description ?? "";
        if (desc.startsWith("Improves ")) {
            desc = desc.replace("Improves ", "") + " Enhancement";
        } else if (desc === "天赋效果增强") {
            desc = "Talent Enhancement";
        } else if (desc === "第一天赋效果增强") {
            desc = "First Talent Enhancement";
        } else if (desc === "第二天赋效果增强") {
            desc = "Second Talent Enhancement";
        }
        return statChanges;
    }

    const attribType = pot.buff.attributes.attributeModifiers?.[0]?.attributeType;
    const attribChange = pot.buff.attributes.attributeModifiers?.[0]?.value;

    switch (attribType) {
        case "MAX_HP":
            statChanges.health += attribChange ?? 0;
            break;
        case "ATK":
            statChanges.attackPower += attribChange ?? 0;
            break;
        case "DEF":
            statChanges.defense += attribChange ?? 0;
            break;
        case "MAGIC_RESISTANCE":
            statChanges.artsResistance += attribChange ?? 0;
            break;
        case "COST":
            statChanges.dpCost += attribChange ?? 0;
            break;
        case "ATTACK_SPEED":
            statChanges.attackSpeed += attribChange ?? 0;
            break;
        case "RESPAWN_TIME":
            statChanges.redeployTimeInSeconds += attribChange ?? 0;
            break;
        default:
            console.warn("Unrecognized attribute in potentials");
            break;
    }

    return statChanges;
};

export const getMaxAttributeStats = (character: CharacterData): Operator["phases"][number]["attributesKeyFrames"][number]["data"] | null => {
    return getCurrentPhase(character)?.attributesKeyFrames[(getCurrentPhase(character)?.attributesKeyFrames ?? []).length - 1]?.data ?? null;
};

export const getCurrentPhase = (character: CharacterData) => {
    const phase = character.evolvePhase;
    return character.static?.phases[phase];
};
