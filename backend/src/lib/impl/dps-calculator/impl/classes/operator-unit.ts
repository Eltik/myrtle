import type { BattleEquip } from "../../../../../types/impl/lib/impl/local/impl/gamedata/impl/modules";
import type { Operator } from "../../../../../types/impl/lib/impl/local/impl/gamedata/impl/operators";
import { getOperatorAttributeStats } from "../helper/getAttributeStats";
import Skills from "./skills";
import Stats from "./stats";

export default class OperatorUnit {
    public stats: Stats;
    public skills: Skills[];
    public operator: Operator;

    constructor(operator: Operator, stats: Stats, skills: Skills[]) {
        this.operator = operator;
        this.stats = stats;
        this.skills = skills;
    }

    public updateAttributeStats(
        metadata: {
            phaseIndex: number;
            favorPoint: number;
            potentialRank: number;
            moduleId: string;
            moduleLevel: number;
        },
        level: number,
        moduleData?: BattleEquip,
    ) {
        const stats = getOperatorAttributeStats(this.operator, metadata, level, moduleData);

        if (!stats) throw new Error("Failed to update attribute stats.");

        this.stats = new Stats(this.operator.name, stats.def, stats.magicResistance, this.operator.profession, this.operator.subProfessionId, stats.maxHp, stats.atk, stats.cost, stats.respawnTime, stats.blockCnt, stats.attackSpeed, null);
    }
}
