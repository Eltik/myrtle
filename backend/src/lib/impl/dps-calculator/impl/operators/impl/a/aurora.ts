import type { OperatorParams } from "../../../../../../../types/impl/lib/impl/dps-calculator";
import OperatorUnit from "../../../classes";
import { OperatorData } from "../../../classes/impl/operator-data";

export default class Aurora extends OperatorUnit {
    constructor(operatorData: OperatorData, params: OperatorParams) {
        super(operatorData, params, 1, 1, -1, [0, 1]);

        this.skillDamageNames = ["1/3vsFreeze"];
        if (this.skillDamage && this.skillIndex === 1) {
            if (this.skillDamageName) {
                this.skillDamageName += " 1/3vsFreeze";
            } else {
                this.skillDamageName = "1/3vsFreeze";
            }
        }
    }

    public skillDPS(enemy: { defense: number; res: number }): number {
        const attackInterval = this.skillIndex === 1 ? 1.85 : this.attackInterval;
        const atkBuff = this.skillIndex === 1 ? this.skillParameters[0] : 0;
        const skillScale = this.skillParameters[3];
        const finalAtk = this.atk * (1 + atkBuff + this.buffATK) + this.buffATKFlat;

        const hitDmg = Math.max(finalAtk - enemy.defense, finalAtk * 0.05);
        const skillDmg = Math.max(finalAtk * skillScale - enemy.defense, finalAtk * skillScale * 0.05);

        let avgDmg = hitDmg;

        if (this.skillDamage && this.skillIndex === 1) {
            avgDmg = (2 / 3) * hitDmg + (1 / 3) * skillDmg;
        }

        const dps = ((avgDmg / attackInterval) * this.attackSpeed) / 100;
        return dps;
    }
}
