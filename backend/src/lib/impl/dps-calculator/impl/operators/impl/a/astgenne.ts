import type { OperatorParams } from "../../../../../../../types/impl/lib/impl/dps-calculator";
import OperatorUnit from "../../../classes";
import { OperatorData } from "../../../classes/impl/operator-data";

export default class Astgenne extends OperatorUnit {
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
    }

    public skillDPS(enemy: { defense: number; res: number }): number {
        let dps = 0;

        const aspd = this.talentDamage && this.elite > 0 ? this.talent1Parameters[0] * this.talent1Parameters[2] : 0;
        const targetScaling = this.elite < 2 ? [0, 1, 1.85, 1.85 + 0.85 ** 2, 1.85 + 0.85 ** 2] : this.operatorModule?.id === "uniequip_002_halo" ? [0, 1, 2, 3, 4] : [0, 1, 1.85, 1.85 + 0.85 ** 2, 1.85 + 0.85 ** 2 + 0.85 ** 3];

        const targets = Math.min(4, this.targets);

        if (this.skillIndex < 1) {
            const skillScale = this.skillParameters[0];
            let spCost = this.skillCost;

            const finalAtk = this.atk * (1 + this.buffATK) + this.buffATKFlat;
            const hitDmg = Math.max(finalAtk * (1 - enemy.res / 100), finalAtk * 0.05) * targetScaling[targets];

            const skillTargetScaling = this.operatorModule?.id === "uniequip_002_halo" ? [0, 1, 4, 6, 8] : [0, 1, 2 * 1.85, 2 * (1.85 + 0.85 ** 2), 2 * (1.85 + 0.85 ** 2 + 0.85 ** 3)];
            const skillDmg = this.skillIndex === -1 ? hitDmg : Math.max(finalAtk * skillScale * (1 - enemy.res / 100), finalAtk * skillScale * 0.05) * skillTargetScaling[targets];

            spCost = spCost / (1 + this.spBoost) + 1.2; // SP lockout

            const atkCycle = this.attackInterval / ((this.attackSpeed + aspd) / 100);
            const attacksPerSkillActivation = spCost / atkCycle;

            let avgHit = skillDmg;
            if (attacksPerSkillActivation > 1 && this.skillIndex === 0) {
                if (this.skillParameters[4] > 1) {
                    avgHit = (skillDmg + (attacksPerSkillActivation - 1) * hitDmg) / attacksPerSkillActivation;
                } else {
                    avgHit = (skillDmg + Math.round(attacksPerSkillActivation) * hitDmg) / (Math.round(attacksPerSkillActivation) + 1);
                }
            }

            dps = ((avgHit / this.attackInterval) * (this.attackSpeed + aspd)) / 100;
        }

        if (this.skillIndex === 1) {
            const finalAtk = this.atk * (1 + this.buffATK + this.skillParameters[0]) + this.buffATKFlat;
            const hitDmg = Math.max(finalAtk * (1 - enemy.res / 100), finalAtk * 0.05);

            dps = ((hitDmg / this.attackInterval) * (this.attackSpeed + aspd)) / 100;

            if (this.targets > 1) {
                dps = (((hitDmg / this.attackInterval) * (this.attackSpeed + aspd)) / 100) * 2 * targetScaling[targets];
            }
        }

        return dps;
    }
}
