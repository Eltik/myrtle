import type { OperatorParams } from "../../../../../../types/impl/lib/impl/dps-calculator";
import OperatorUnit from "../../classes";
import { OperatorData } from "../../classes/impl/operator-data";

export default class AmiyaGuard extends OperatorUnit {
    constructor(operatorData: OperatorData, params: OperatorParams) {
        super(operatorData, params, 0, 6, 1); // TODO: Fix

        this.skillDamageNames = ["3kills"];
        if (this.skillIndex === 1) {
            if (this.skillDamage) {
                this.skillDamageName += " 3kills";
            } else {
                this.skillDamageName = "3kills";
            }
        }
    }

    public skillDPS(enemy: { defense: number; res: number }): number {
        let dps = 0;
        let atkBuff = this.talent1Parameters[0] * (1 + Math.min(1, this.skillIndex + 1));

        if (this.skillIndex === 1) {
            atkBuff += this.skillParameters[0] * (this.skillIndex + 1);

            const finalAtk = this.atk * (1 + atkBuff + this.buffATK) + this.buffATKFlat;
            const hitDmgArts = Math.max(finalAtk * (1 - enemy.res / 100), finalAtk * 0.05);

            dps = ((((1 + (this.skillIndex + 1)) * hitDmgArts) / this.attackInterval) * this.attackSpeed) / 100;
        }

        if (this.skillIndex === 0) {
            if (this.skillDamage) {
                atkBuff += 3 * this.skillParameters[3];
            }

            const finalAtk = this.atk * (1 + atkBuff + this.buffATK) + this.buffATKFlat;

            dps = (((finalAtk / this.attackInterval) * this.attackSpeed) / 100) * Math.max(1, -enemy.defense);
        }

        return dps;
    }
}
