import type { OperatorParams } from "../../../../../../types/impl/lib/impl/dps-calculator";
import OperatorUnit from "../../classes";
import { OperatorData } from "../../classes/impl/operator-data";

export default class Absinthe extends OperatorUnit {
    constructor(operatorData: OperatorData, params: OperatorParams) {
        super(operatorData, params, 1, 6, 1);

        if (this.skillIndex === 1 && this.operatorModule && this.operatorModuleLevel > 1) {
            this.talentDamage = true;
        }

        this.talentDamageNames = ["lowHpTarget"];
        if (this.talentDamage && this.elite > 0) {
            if (this.talentDamageName) {
                this.talentDamageName += " lowHpTarget";
            } else {
                this.talentDamageName = "lowHpTarget";
            }
        }
    }

    public skillDPS(enemy: { defense: number; res: number }): number {
        const dmgScale = this.talentDamage && this.elite > 0 ? this.talent1Parameters[1] : 1;

        const res = this.operatorModule ? Math.max(0, enemy.res - 10) : enemy.res;
        const finalAtk = this.skillIndex === 0 ? this.atk * (1 + this.skillParameters[0] + this.buffATK) + this.buffATKFlat : this.atk * (1 + this.buffATK) + this.buffATKFlat;
        const atkScale = this.skillIndex === 1 ? 4 * this.skillParameters[1] : 1;
        const hitDmgArts = Math.max(finalAtk * atkScale * (1 - res / 100), finalAtk * atkScale * 0.05) * dmgScale;
        const dps = ((hitDmgArts / this.attackInterval) * this.attackSpeed) / 100;
        return dps;
    }
}
