import { Operator } from "../local/impl/gamedata/impl/operators";

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
