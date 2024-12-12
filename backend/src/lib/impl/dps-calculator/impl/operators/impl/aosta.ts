import type { OperatorParams } from "../../../../../../types/impl/lib/impl/dps-calculator";
import OperatorUnit from "../../classes";
import { OperatorData } from "../../classes/impl/operator-data";

export default class Aosta extends OperatorUnit {
    constructor(operatorData: OperatorData, params: OperatorParams) {
        super(operatorData, params, 1, 1, -1);

        this.traitDamageNames = ["distant"];
        if (!this.traitDamage) {
            if (this.traitDamageName) {
                this.traitDamageName += " distant";
            } else {
                this.traitDamageName = "distant";
            }
        }

        this.talentDamageNames = ["blockTarget"];
        if (this.elite > 0 && !this.talentDamage) {
            if (this.talentDamageName) {
                this.talentDamageName += " blockedTarget";
            } else {
                this.talentDamageName = "blockedTarget";
            }
        }
    }

    public skillDPS(enemy: { defense: number; res: number }): number {
        const atkScale = this.traitDamage ? 1.5 : 1;
        let talentScale = this.elite > 0 && this.talentDamage ? this.talent1Parameters[0] : 0;
        const talentDuration = this.talent1Parameters[1];
        const aspd = this.skillIndex === 0 ? this.skillParameters[1] : 0;

        let dps = 0;
        let finalAtk = 0;
        if (this.skillIndex === 0) {
            finalAtk = this.atk * (1 + this.skillParameters[0] * (this.skillIndex + 1) + this.buffATK) + this.buffATKFlat;
            const hitDmg = Math.max(finalAtk * atkScale - enemy.defense, finalAtk * atkScale * 0.05);

            dps = (((hitDmg / this.attackInterval) * (this.attackSpeed + aspd)) / 100) * this.targets;
        }
        if (this.skillIndex === 1) {
            finalAtk = this.atk * (1 + this.skillParameters[1] + this.buffATK) + this.buffATKFlat;
            const hitDmg = Math.max(finalAtk * atkScale - enemy.defense, finalAtk * atkScale * 0.05);

            dps = (((hitDmg / 3.45) * this.attackSpeed) / 100) * this.targets;
            talentScale *= 2;
        }

        const activeRatio = Math.min(1, talentDuration / ((this.attackInterval / (this.attackSpeed + aspd)) * 100));
        const artsDPS = Math.max(finalAtk * talentScale * (1 - enemy.res / 100), finalAtk * talentScale * 0.05) * activeRatio * this.targets;
        dps += artsDPS;
        return dps;
    }
}
