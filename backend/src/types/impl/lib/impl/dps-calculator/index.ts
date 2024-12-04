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
    physical = 0,
    arts = 1,
    true = 2,
    healing = 3,
}
