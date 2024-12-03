/* eslint-disable @typescript-eslint/no-redundant-type-constituents */
/* eslint-disable @typescript-eslint/no-explicit-any */
import type { Operator } from "../../api/static/operator";

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

export type CalculateDPSParams = {
    char: Operator;
    options: {
        buff?: boolean;
        cond?: boolean;
        cond_2?: boolean;
        cond_front?: boolean;
        cond_def?: boolean;
        cond_spd?: boolean;
        cond_near?: boolean;
        cond_elite?: boolean;
        stack?: boolean;
        ranged_penalty?: boolean;
        freeze?: boolean;
        block?: boolean;
        token?: boolean;
        pallas?: boolean;
        charge?: boolean;
        warmup?: boolean;
        rosmon_double?: boolean;
        equip?: boolean;
        annie?: boolean;
        noblock?: boolean;
        ling_fusion?: boolean;
        water?: boolean;
        melee?: boolean;
        far?: boolean;
        overdrive_mode?: boolean;
        crit?: boolean;
        od_trigger?: boolean;
        reed2_fast?: boolean;
        thorns_ranged?: boolean;
        archet?: boolean;
        chen?: boolean;
        short_mode?: boolean;
        cannon?: boolean;
    };
    operatorData: {
        level: number;
        phaseIndex: number;
        favor: number;
        potentialRank: number;
        equipId: string;
        equipLevel: number;
        skillId: string;
        skillIndex: number;
    };
    enemy: {
        def: number;
        res: number;
        count: number;
    };
    buffConfig: {
        atk: number;
        atkpct: number;
        ats: number;
        cdr: number;
        base_atk: number;
        damage_scale: number;
    };
};

export type CharAttr = {
    basic: Record<string, number>;
    buffs: Record<string, number>;
    buffList: BuffList;
    char: Operator;
};

export type BuffList = {
    skill: SkillBuff;
    [key: string]: SkillBuff | any; // Allow for other buff types
};

export type SkillBuff = {
    id: string;
    talent_scale?: number;
    ["talent@prob"]?: number;
    trait_up?: number;
};

export type BuffFrame = {
    atk_scale: number;
    def_scale: number;
    heal_scale: number;
    damage_scale: number;
    maxTarget: number;
    times: number;
    edef: number;
    edef_scale: number;
    edef_pene: number;
    edef_pene_scale: number;
    emr_pene: number;
    emr: number;
    emr_scale: number;
    atk: number;
    def: number;
    attackSpeed: number;
    maxHp: number;
    baseAttackTime: number;
    spRecoveryPerSec: number;
    spRecoverRatio: number;
    spRecoverIntervals: any[]; // This could be further refined
    applied: Record<string, boolean>;
    [key: string]: any; // Allow for additional properties
};

export type Enemy = {
    def: number;
    res: number;
    count: number;
};
