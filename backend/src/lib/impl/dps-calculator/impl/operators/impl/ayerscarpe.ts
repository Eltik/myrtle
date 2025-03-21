import type { OperatorParams } from "../../../../../../types/impl/lib/impl/dps-calculator";
import OperatorUnit from "../../classes";
import { OperatorData } from "../../classes/impl/operator-data";

export default class Ayerscarpe extends OperatorUnit {
    constructor(operatorData: OperatorData, params: OperatorParams) {
        super(operatorData, params, 1, 1, 1, [0, 1]);

        this.skillDamageNames = ["rangedAtk"];
        if (!this.traitDamage && this.skillIndex === 0) {
            if (this.skillDamageName) {
                this.skillDamageName += " rangedAtk";
            } else {
                this.skillDamageName = "rangedAtk";
            }
        }
    }

    public skillDPS(enemy: { defense: number; res: number }): number {
        let dps = 0;

        const atkScale = !this.traitDamage && this.skillIndex === 0 ? 0.8 : 1;
        const bonus = this.operatorModule?.id === "uniequip_002_ayer" ? 0.1 : 0;
        const aspd = this.elite > 0 ? this.talent1Parameters[0] : 0;
        const finalAtk = this.atk * (1 + this.buffATK) + this.buffATKFlat;

        if (this.skillIndex < 1) {
            const skillScale = this.skillParameters[0];
            const targets = this.skillIndex === 1 ? this.skillParameters[2] : 1;

            const hitDmg = Math.max(finalAtk * atkScale - enemy.defense, finalAtk * atkScale * 0.05);
            const bonusDmg = Math.max(finalAtk * bonus * (1 - enemy.res / 100), finalAtk * bonus * 0.05);
            const skillDmg = Math.max(finalAtk * skillScale * (1 - enemy.res / 100), finalAtk * skillScale * 0.05) * Math.min(this.targets, targets);

            const avgDmg = this.skillIndex === -1 ? hitDmg : (this.skillCost * hitDmg + skillDmg) / (this.skillCost + 1);
            dps = (((avgDmg + bonusDmg) / this.attackInterval) * (this.attackSpeed + aspd)) / 100;
        }
        if (this.skillIndex === 1) {
            const skillScale = this.skillParameters[1];

            const hitDmg = Math.max(finalAtk * atkScale * (1 - enemy.res / 100), finalAtk * atkScale * 0.05);
            const bonusDmg = Math.max(finalAtk * bonus * (1 - enemy.res / 100), finalAtk * bonus * 0.05);
            let skillDmg = Math.max(finalAtk * skillScale * (1 - enemy.res / 100), finalAtk * skillScale * 0.05);

            if (this.traitDamage) {
                skillDmg *= this.targets;
            } else {
                skillDmg *= this.targets - 1;
            }

            dps = (((hitDmg + bonusDmg + skillDmg) / this.attackInterval) * (this.attackSpeed + aspd)) / 100;
        }

        return dps;
    }
}
