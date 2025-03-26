import type { OperatorParams } from "../../../../../../../types/impl/lib/impl/dps-calculator";
import OperatorUnit from "../../../classes";
import { OperatorData } from "../../../classes/impl/operator-data";

export default class Andreana extends OperatorUnit {
    constructor(operatorData: OperatorData, params: OperatorParams) {
        super(operatorData, params, 0, 1, 1, [0, 1]);

        this.talentDamageNames = ["atMaxRange"];
        if (this.operatorModule?.id === "uniequip_002_cuttle" && this.moduleDamage) {
            if (this.talentDamageName) {
                this.talentDamageName += " atMaxRange";
            } else {
                this.talentDamageName = "atMaxRange";
            }
        }
    }

    public skillDPS(enemy: { defense: number; res: number }): number {
        const atkScale = this.operatorModule?.id === "uniequip_002_cuttle" && this.moduleDamage ? 1.15 : 1;

        const finalAtk = this.atk * (1 + this.buffATK + this.skillParameters[0] * Math.min(this.skillIndex + 1, 1)) + this.buffATKFlat;
        const hitDmg = Math.max(finalAtk * atkScale - enemy.defense, finalAtk * atkScale * 0.05);

        const dps = ((hitDmg / this.attackInterval) * (this.attackSpeed + this.talent1Parameters[0])) / 100;
        return dps;
    }
}
