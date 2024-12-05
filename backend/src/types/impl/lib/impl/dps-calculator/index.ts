import type { Operator } from "../local/impl/gamedata/impl/operators";
import { Skill } from "../local/impl/gamedata/impl/skills";

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
    physical = 0,
    arts = 1,
    true = 2,
    healing = 3,
}

export type SkillData = Omit<Skill, "levels"> & {
    levels: (Skill["levels"][number] & { modifiers: Modifiers })[];
};

export type Modifiers = {
    // Attacks_Per_Second
    attackSpeedModifiers?: number[];
    attackIntervalModifiers?: number[];

    // Final_Attack
    baseAttackModifiers?: number[];
    attackMultiplierModifiers?: number[];
    soraBuff?: number[]; // We love Sora

    // Physical_Damage
    flatDefModifiers?: number[];
    scalingDefModifiers?: number[];
    physTakenModifiers?: number[];
    extraPhysDamageDone?: number[];

    // Arts_Damage
    flatResModifiers?: number[];
    scalingResModifiers?: number[];
    artsTakenModifiers?: number[];
    extraArtsDamageDone?: number[];

    // True_Damage
    extraTrueDamageDone?: number[];
};
