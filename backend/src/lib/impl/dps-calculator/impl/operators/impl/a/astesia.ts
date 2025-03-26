import type { OperatorParams } from "../../../../../../../types/impl/lib/impl/dps-calculator";
import OperatorUnit from "../../../classes";
import { OperatorData } from "../../../classes/impl/operator-data";

export default class Astesia extends OperatorUnit {
    constructor(operatorData: OperatorData, params: OperatorParams) {
        super(operatorData, params, 1, 1, -1, [0, 1]);

        this.talentDamageNames = ["maxStacks"];
        if (this.talentDamage) {
            if (this.talentDamageName) {
                this.talentDamageName += " maxStacks";
            } else {
                this.talentDamageName = "maxStacks";
            }
        }

        /**
         * @todo: Add module when arts fighter gets one
         */
    }

    public skillDPS(enemy: { defense: number; res: number }): number {
        const dmg = 1; // Eventually add module damage

        const aspd = this.talentDamage ? this.talent1Parameters[0] * this.talent1Parameters[2] : 0;
        const atkBuff = this.skillIndex > -1 ? this.skillParameters[0] : 0;

        const finalAtk = this.atk * (1 + this.buffATK + atkBuff) + this.buffATKFlat;
        const hitDmg = Math.max(finalAtk * (1 - enemy.res / 100), finalAtk * 0.05);

        let dps = (((hitDmg / this.attackInterval) * (this.attackSpeed + aspd)) / 100) * dmg;

        if (this.skillIndex === 1) {
            dps *= Math.min(this.targets, 2);
        }

        return dps;
    }
}
