import type { OperatorParams } from "../../../../../../types/impl/lib/impl/dps-calculator";
import OperatorUnit from "../../classes";
import { OperatorData } from "../../classes/impl/operator-data";

export default class Adnachiel extends OperatorUnit {
    constructor(operatorData: OperatorData, params: OperatorParams) {
        super(operatorData, params, 0, 6, 0);
    }

    public skillDPS(enemy: { defense: number; res: number }): number {
        const aspd = this.talent1Parameters[0];

        const finalAtk = this.atk * (1 + this.skillParameters[0] * (this.skillIndex + 1) + this.buffATK) + this.buffATKFlat;
        const hitDmg = Math.max(finalAtk - enemy.defense, finalAtk * 0.05);

        const dps = ((hitDmg / this.attackInterval) * (this.attackSpeed + aspd)) / 100;
        return dps;
    }
}
