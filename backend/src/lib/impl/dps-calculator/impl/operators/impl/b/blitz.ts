import type { OperatorParams } from "../../../../../../../types/impl/lib/impl/dps-calculator";
import OperatorUnit from "../../../classes";
import { OperatorData } from "../../../classes/impl/operator-data";

export default class Blitz extends OperatorUnit {
    constructor(operatorData: OperatorData, params: OperatorParams) {
        super(operatorData, params, 1, 1, 1, [0, 1]);

        this.talentDamageNames = ["vsStun"];
        if (this.skillIndex === 0 && this.talentDamage) {
            if (this.talentDamageName) {
                this.talentDamageName += " vsStun";
            } else {
                this.talentDamageName = "vsStun";
            }
        }
    }

    public skillDPS(enemy: { defense: number; res: number }): number {
        let dps = 0;
        let atkScale = this.skillIndex < 1 && !this.talentDamage ? 1 : this.talent1Parameters[0];

        if (this.skillIndex < 1) {
            const finalAtk = this.atk * (1 + this.buffATK) + this.buffATKFlat;
            const hitDmg = Math.max(finalAtk * atkScale - enemy.defense, finalAtk * atkScale * 0.05);
            dps = ((hitDmg / this.attackInterval) * this.attackSpeed) / 100;
        }

        if (this.skillIndex === 1) {
            const finalAtk = this.atk * (1 + this.buffATK) + this.buffATKFlat;
            atkScale -= 1;
            atkScale *= this.skillParameters[3];
            atkScale += 1;

            const hitDmg = Math.max(finalAtk * atkScale - enemy.defense, finalAtk * atkScale * 0.05);
            dps = ((hitDmg / this.attackInterval) * (this.attackSpeed + 200)) / 100;
        }

        return dps;
    }
}
