import type { OperatorParams } from "../../../../../../types/impl/lib/impl/dps-calculator";
import type { Operator } from "../../../../../../types/impl/lib/impl/local/impl/gamedata/impl/operators";
import OperatorUnit from "../../classes";

export default class TwelveF extends OperatorUnit {
    constructor(operatorData: Operator, params: OperatorParams) {
        super(operatorData, params, -1, 6, -1);

        if (this.potential > 2) this.atk += 12;
    }

    public skillDPS(enemy: { defense: number; res: number }): number {
        const finalAtk = this.atk * (1 + this.buffATK) + this.buffATKFlat;
        const hitDmg = Math.max(finalAtk * (1 - enemy.res / 100), finalAtk * 0.05);
        const dps = (((hitDmg / this.attackInterval) * this.attackSpeed) / 100) * this.targets;
        return dps;
    }
}
