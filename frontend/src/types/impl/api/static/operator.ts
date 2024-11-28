import type { Item, ItemType } from "./material";

export type Operator = {
    id?: string;
    name: string;
    description: string;
    canUseGeneralPotentialItem: boolean;
    canUseActivityPotentialItem: boolean;
    potentialItemId: string;
    activityPotentialItemId: string | null;
    classicPotentialItemId: string | null;
    nationId: string;
    groupId: string | null;
    teamId: string | null;
    displayNumber: string;
    appellation: string;
    position: OperatorPosition;
    tagList: string[];
    itemUsage: string;
    itemDesc: string;
    itemObtainApproach: string;
    isNotObtainable: boolean;
    isSpChar: boolean;
    maxPotentialLevel: number;
    rarity: OperatorRarity;
    profession: OperatorProfession;
    subProfessionId: string;
    trait: {
        candidates: {
            unlockCondition: {
                phase: OperatorPhase;
                level: number;
            };
            requiredPotentialRank: number;
            blackboard: {
                key: string;
                value: number;
                valueStr: string | null;
            }[];
            overrideDescripton: string | null;
            prefabKey: string | null;
            rangeId: string | null;
        }[];
    } | null;
    phases: {
        characterPrefabKey: string;
        rangeId: string;
        maxLevel: number;
        attributesKeyFrames: {
            level: number;
            data: {
                maxHp: number;
                atk: number;
                def: number;
                magicResistance: number;
                cost: number;
                blockCnt: number;
                moveSpeed: number;
                attackSpeed: number;
                baseAttackTime: number;
                respawnTime: number;
                hpRecoveryPerSec: number;
                spRecoveryPerSec: number;
                maxDeployCount: number;
                maxDeckStackCnt: number;
                tauntLevel: number;
                massLevel: number;
                baseForceLevel: number;
                stunImmune: boolean;
                silenceImmune: boolean;
                sleepImmune: boolean;
                frozenImmune: boolean;
                levitateImmune: boolean;
                disarmedCombatImmune: boolean;
            };
        }[];
        evolveCost:
            | {
                  id: string;
                  count: number;
                  type: ItemType;
              }[]
            | null;
    }[];
    skills: {
        skillId: string;
        overridePrefabKey: string | null;
        overrideTokenKey: string | null;
        levelUpCostCond: {
            unlockCond: {
                phase: OperatorPhase;
                level: number;
            };
            lvlUpTime: number;
            levelUpCost: {
                id: string;
                count: number;
                type: string;
            }[];
        }[];
        // Obtained from the skills lib
        static?: {
            levels: {
                name: string;
                rangeId: string | null;
                description: string;
                skillType: string;
                duration: number;
                spData: {
                    spType: string;
                    levelUpCost: [];
                    maxChargeTime: number;
                    spCost: number;
                    initSp: number;
                    increment: number;
                };
                prefabId: string;
                blackboard: { key: string; value: number; valueStr: string | null }[];
            }[];
            skillId: string;
            iconId: string | null;
            hidden: boolean;
            image: string | null;
        };
    }[];
    displayTokenDict: null;
    talents: {
        candidates: {
            unlockCondition: {
                phase: OperatorPhase;
                level: number;
            };
            requiredPotentialRank: number;
            prefabKey: string | null;
            name: string;
            description: string;
            rangeId: string | null;
            blackboard: {
                key: string;
                value: number;
                valueStr: string | null;
            }[];
            tokenKey: string | null;
        }[];
    }[];
    potentialRanks: {
        type: string;
        description: string;
        buff: {
            attributes: {
                abnormalFlags: null;
                abnormalImmunes: null;
                abnormalAntis: null;
                abnormalCombos: null;
                abnormalComboImmunes: null;
                attributeModifiers:
                    | {
                          attributeType: string;
                          formulaItem: string;
                          value: number;
                          loadFromBlackboard: boolean;
                          fetchBaseValueFromSourceEntity: boolean;
                      }[]
                    | null;
            };
        };
    }[];
    favorKeyFrames: {
        level: number;
        data: {
            maxHp: number;
            atk: number;
            def: number;
            magicResistance: number;
            cost: number;
            blockCnt: number;
            moveSpeed: number;
            attackSpeed: number;
            baseAttackTime: number;
            respawnTime: number;
            hpRecoveryPerSec: number;
            spRecoveryPerSec: number;
            maxDeployCount: number;
            maxDeckStackCnt: number;
            tauntLevel: number;
            massLevel: number;
            baseForceLevel: number;
            stunImmune: boolean;
            silenceImmune: boolean;
            sleepImmune: boolean;
            frozenImmune: boolean;
            levitateImmune: boolean;
            disarmedCombatImmune: boolean;
        };
    }[];
    allSkillLevelUp: {
        unlockCond: {
            phase: OperatorPhase;
            level: number;
        };
        lvlUpCost: {
            id: string;
            count: number;
            type: string;
        }[];
    }[];
};

export enum OperatorPosition {
    RANGED = "RANGED",
    MELEE = "MELEE",
}

export enum OperatorPhase {
    ELITE_0 = "PHASE_0",
    ELITE_1 = "PHASE_1",
    ELITE_2 = "PHASE_2",
}

export enum OperatorRarity {
    sixStar = "TIER_6",
    fiveStar = "TIER_5",
    fourStar = "TIER_4",
    threeStar = "TIER_3",
    twoStar = "TIER_2",
    oneStar = "TIER_1",
}

export enum OperatorProfession {
    MEDIC = "MEDIC",
    CASTER = "CASTER",
    GUARD = "WARRIOR",
    VANGUARD = "PIONEER",
    SNIPER = "SNIPER",
    SPECIALIST = "SPECIAL",
    SUPPORTER = "SUPPORT",
    DEFENDER = "TANK",
}

export type LevelCost = {
    level: number;
    eliteCost: { quantity: number; material: Item }[];
    elite: "E0" | "E1" | "E2";
};
export type LevelUpCost = LevelCost[];

export type SkillCost = {
    unlockCondition: {
        phase: OperatorPhase;
        level: number;
    };
    lvlUpTime: number;
    skillCost: { quantity: number; material: Item }[];
    level: "M1" | "M2" | "M3";
};

export type SkillLevelCost = {
    skillId: string;
    cost: SkillCost[];
};

export type SkillLevelUpCost = SkillLevelCost[];
