import type { OperatorParams } from "../../../../../../../types/impl/lib/impl/dps-calculator";
import OperatorUnit from "../../../classes";
import { OperatorData } from "../../../classes/impl/operator-data";

export default class Aciddrop extends OperatorUnit {
    constructor(operatorData: OperatorData, params: OperatorParams) {
        super(operatorData, params, 1, 6, 1, [0, 1]);

        this.talentDamageNames = ["directFront"];
        if (this.talentDamage && this.elite > 0) {
            if (this.talentDamageName) {
                this.talentDamageName += " directFront";
            } else {
                this.talentDamageName = "directFront";
            }
        }
    }

    public skillDPS(enemy: { defense: number; res: number }): number {
        let minDmg = 0;
        if (this.elite === 0) {
            minDmg = 0.05;
        } else if (this.talentDamage) {
            minDmg = this.talent1Parameters[1];
        } else {
            minDmg = this.talent1Parameters[0];
        }

        const aspd = this.skillIndex === 0 ? this.skillParameters[0] : 0;
        const atkBuff = this.skillIndex === 1 ? this.skillParameters[0] : 0;

        const finalAtk = this.atk * (1 + this.buffATK + atkBuff) + this.buffATKFlat;
        const hitDmg = Math.max(finalAtk - enemy.defense, finalAtk * minDmg);

        const dps = (((hitDmg / this.attackInterval) * (this.attackSpeed + aspd)) / 100) * Math.max(1, this.skillIndex + 1);
        return dps;
    }
}
