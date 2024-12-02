import { Operator } from "../local/impl/gamedata/impl/operators";

export type CalculatorParams = {
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
