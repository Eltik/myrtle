import type { OperatorParams } from "../../../../../../../types/impl/lib/impl/dps-calculator";
import OperatorUnit from "../../../classes";
import { OperatorData } from "../../../classes/impl/operator-data";

export default class Beeswax extends OperatorUnit {
    constructor(operatorData: OperatorData, params: OperatorParams) {
        super(operatorData, params, 1, 1, 1, [0, 1]);

        this.skillDamageNames = [];
        if (this.skillIndex === 1) {
            const finalAtk = this.atk * (1 + this.buffATK) + this.buffATKFlat;
            const nukeHit = finalAtk * this.skillParameters[0];

            const skillDamageName = `InitialHit: ${Math.round(nukeHit)}`;
            this.skillDamageNames.push(skillDamageName);

            if (this.skillDamageName) {
                this.skillDamageName += ` ${skillDamageName}`;
            } else {
                this.skillDamageName = skillDamageName;
            }
        }
    }

    public skillDPS(enemy: { defense: number; res: number }): number {
        if (this.skillIndex === -1) return enemy.res * 0;

        const atkBuff = this.skillIndex === 0 ? this.skillParameters[0] : 0;
        const finalAtk = this.atk * (1 + atkBuff + this.buffATK) + this.buffATKFlat;
        const hitDmg = Math.max(finalAtk * (1 - enemy.res / 100), finalAtk * 0.05);

        const dps = ((hitDmg / this.attackInterval) * this.attackSpeed) / 100;
        return dps;
    }
}
