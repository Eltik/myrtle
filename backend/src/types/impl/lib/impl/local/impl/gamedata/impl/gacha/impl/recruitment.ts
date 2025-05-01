import type { GachaTag } from "..";
import type { Operator } from "../../operators";

export type RecruitGroup = {
    tags: GachaTag[];
    matches: Operator[];
    lowest9hrRarity: number;
    highestRarity: number;
    nineHourOpCount: number;
};
