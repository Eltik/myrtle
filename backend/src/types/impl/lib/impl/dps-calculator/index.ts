import { Enemy } from "../local/impl/gamedata/impl/enemies";
import type { Operator } from "../local/impl/gamedata/impl/operators";

export type CalculateNormalATKParams = {
    operatorPhase: Operator["phases"][0]["attributesKeyFrames"][0];
    buffs: {
        attack: number;
        attackFlat: number;
    };
    isPhysical: boolean;
    enemy: {
        defense: number;
        res: number;
    };
    extraBuffs?: [number, number, number];
    hits?: number;
    aoe?: number;
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
