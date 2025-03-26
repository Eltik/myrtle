import type { OperatorParams } from "../../../../../../../types/impl/lib/impl/dps-calculator";
import OperatorUnit from "../../../classes";
import { OperatorData } from "../../../classes/impl/operator-data";

export default class Beehunter extends OperatorUnit {
    constructor(operatorData: OperatorData, params: OperatorParams) {
        super(operatorData, params, 1, 6, 1, [0, 1]);

        this.talentDamageNames = ["maxStacks"];
        if (this.talentDamage && this.elite > 0) {
            if (this.talentDamageName) {
                this.talentDamageName += " maxStacks";
            } else {
                this.talentDamageName = "maxStacks";
            }
        }

        this.moduleDamageNames = [">50% hp", "<50% hp"];
        if (this.operatorModule?.id === "uniequip_002_brownb") {
            if (this.moduleDamage) {
                this.moduleDamageName = ">50% hp";
            } else {
                this.moduleDamageName = "<50% hp";
            }
        }
    }

    public skillDPS(enemy: { defense: number; res: number }): number {
        const atkBuff = this.talentDamage ? this.talent1Parameters[0] * this.talent1Parameters[1] : 0;
        const aspd = this.moduleDamage && this.operatorModule?.id === "uniequip_002_brownb" ? 10 : 0;

        const atkInterval = this.skillIndex === 1 ? this.attackInterval * (1 + this.skillParameters[0]) : this.attackInterval;
        const finalAtk = this.atk * (1 + atkBuff) + this.buffATKFlat;
        const hitDmg = Math.max(finalAtk - enemy.defense, finalAtk * 0.05);

        const dps = ((hitDmg / atkInterval) * (this.attackSpeed + aspd)) / 100;
        return dps;
    }
}
