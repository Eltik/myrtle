import type { OperatorParams } from "../../../../../../../types/impl/lib/impl/dps-calculator";
import OperatorUnit from "../../../classes";
import { OperatorData } from "../../../classes/impl/operator-data";

export default class Asbestos extends OperatorUnit {
    constructor(operatorData: OperatorData, params: OperatorParams) {
        super(operatorData, params, 1, 1, -1, [0, 1]);
    }

    public skillDPS(enemy: { defense: number; res: number }): number {
        let dps = 0;

        if (this.skillIndex === -1) {
            const finalAtk = this.atk * (1 + this.buffATK) + this.buffATKFlat;
            const hitDmg = Math.max(finalAtk - enemy.defense, finalAtk * 0.05);

            dps = ((hitDmg / this.attackInterval) * this.attackSpeed) / 100;
        }
        if (this.skillIndex === 0) {
            const finalAtk = this.atk * (1 + this.buffATK) + this.buffATKFlat;
            const hitDmg = Math.max(finalAtk * (1 - enemy.res / 100), finalAtk * 0.05);

            dps = ((hitDmg / this.attackInterval) * this.attackSpeed) / 100;
        }
        if (this.skillIndex === 1) {
            const finalAtk = this.atk * (1 + this.skillParameters[0] + this.buffATK) + this.buffATKFlat;
            const hitDmg = Math.max(finalAtk * (1 - enemy.res / 100), finalAtk * 0.05);

            dps = (((hitDmg / 2) * this.attackSpeed) / 100) * this.targets;
        }

        return dps;
    }
}
