import type { Enemy } from "../static/enemies";
import type { Module } from "../static/modules";
import type { Operator } from "../static/operator";

export type DPSCalculatorResponse = {
    dps: number;
    operator: DPSOperator;
};

export type DPSOperatorResponse = {
    operator: DPSOperator;
};

export type DPSOperator = {
    atk: number;
    attackInterval: number;
    attackSpeed: number;
    buffATK: number;
    buffATKFlat: number;
    buffFragile: number;
    buffName: string;
    defaultSkillIndex: number;
    defaultModuleIndex: number;
    defaultPotential: number;
    defaultPromotion: number;
    traitDamageNames: string[];
    talentDamageNames: string[];
    talent2DamageNames: string[];
    skillDamageNames: string[];
    moduleDamageNames: string[];
    droneAtk: number;
    droneAtkInterval: number;
    elite: number;
    isPhysical: boolean;
    isRanged: boolean;
    level: number;
    moduleDamage: boolean;
    moduleDamageName: string;
    operatorData: {
        aspdModule: {
            moduleId: string;
            value: number;
        }[];
        aspdPotential: {
            requiredPotential: number;
            value: number;
        };
        aspdTrust: number;
        atk: {
            e0: {
                min: number;
                max: number;
            };
            e1: {
                min: number;
                max: number;
            };
            e2: {
                min: number;
                max: number;
            };
        };
        atkModule: {
            moduleId: string;
            value: number;
        }[];
        atkPotential: {
            requiredPotential: number;
            value: number;
        };
        atkTrust: number;
        attackInterval: number;
        availableModules: Module[];
        data: Operator;
        droneAtk: {
            e0: {
                min: number;
                max: number;
            };
            e1: {
                min: number;
                max: number;
            };
            e2: {
                min: number;
                max: number;
            };
        };
        droneAtkInterval: number[];
        hasSecondTalent: boolean;
        isPhysical: boolean;
        isRanged: boolean;
        rarity: number;
        skillCosts: number[][];
        skillDurations: number[][];
        skillParameters: number[][];
        talent1Defaults: number[];
        talent1ModuleExtra: number[];
        talent1Parameters: number[][];
        talent2Defaults: number[];
        talent2ModuleExtra: number[];
        talent2Parameters: number[][];
    };
    operatorModule: Module;
    operatorModuleLevel: number;
    params: OperatorParams;
    potential: number;
    rarity: number;
    skillCost: number;
    skillDamage: boolean;
    skillDuration: number;
    skillIndex: number;
    skillLevel: number;
    skillParameters: number[];
    spBoost: number;
    talent1Parameters: number[];
    talent2Damage: boolean;
    talent2Parameters: number[];
    talentDamage: boolean;
    targets: number;
    traitDamage: boolean;
    trust: number;
};

export enum AttackType {
    PHYSICAL = 0,
    ARTS = 1,
    TRUE = 2,
}

export type OperatorParams = {
    potential?: number;
    promotion?: number;
    level?: number;
    trust?: number;

    skillIndex?: number;
    masteryLevel?: number;

    moduleIndex?: number;
    moduleLevel?: number;

    /**
     * 0: ATK buff as a percentage decimal (eg. 0.4)
     * 1: Flat ATK buff (eg. 102)
     * 2: ASPD buff (eg. 52)
     * 3: Fragile debuff as a percentage decimal (eg. 0.3)
     */
    buffs?: {
        atk?: number;
        atkFlat?: number;
        aspd?: number;
        fragile?: number;
    };

    /**
     * 0: ATK buff as a percentage decimal (eg. 0.4)
     * 1: Flat ATK buff (eg. 102)
     */
    baseBuffs?: {
        atk?: number;
        atkFlat?: number;
    };

    spBoost?: number;

    targets?: number;
    enemies?: Enemy[];

    conditionals?: {
        traitDamage?: boolean;

        talentDamage?: boolean;
        talent2Damage?: boolean;

        skillDamage?: boolean;

        moduleDamage?: boolean;
    };
    allCond?: boolean;

    graphType?: number;
    fixValue?: number;

    maxDef?: number;
    maxRes?: number;
    res?: [number];
    def?: [number];

    /**
     * 0: DEF shred as a percentage decimal (eg. 0.4)
     * 1: Flat DEF shred (eg. 102)
     * 2: RES shred as a percentage decimal (eg. 0.4)
     * 3: Flat RES shred (eg. 102)
     */
    shred?: {
        def?: number;
        defFlat?: number;
        res?: number;
        resFlat?: number;
    };

    normalDPS?: number;
};

export type OperatorTalentParameter = {
    requiredPromotion: number;
    requiredLevel: number;
    requiredModuleId: string;
    requiredModuleLevel: number;
    requiredPotential: number;
    talentData: number[];
};
