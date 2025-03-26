import type { OperatorParams } from "../../../../../../types/impl/lib/impl/dps-calculator";
import OperatorUnit from "../../classes";
import { OperatorData } from "../../classes/impl/operator-data";

export default class Bagpipe extends OperatorUnit {
    constructor(operatorData: OperatorData, params: OperatorParams) {
        super(operatorData, params, 2, 1, 1, [0, 1, 2]);

        this.moduleDamageNames = ["lowHpTarget"];
        if (this.moduleDamage && this.operatorModule?.id === "uniequip_003_bpipe") {
            if (this.moduleDamageName) {
                this.moduleDamageName += " lowHpTarget";
            } else {
                this.moduleDamageName = "lowHpTarget";
            }
        }
    }

    public skillDPS(enemy: { defense: number; res: number }): number {
        let dps = 0;

        const atkScale = this.moduleDamage && this.operatorModule?.id === "uniequip_003_bpipe" ? 1.15 : 1;
        const cRate = this.elite > 0 ? this.talent1Parameters[1] : 0;
        const cDMG = this.elite > 0 ? this.talent1Parameters[0] : 1;

        if (this.skillIndex === 0) {
            const atkBuff = this.skillParameters[0] * (this.skillIndex + 1);
            const aspd = this.skillParameters[1] * (this.skillIndex + 1);

            const finalAtk = this.atk * (1 + atkBuff + this.buffATK) + this.buffATKFlat;
            const hitDmg = Math.max(finalAtk * atkScale - enemy.defense, finalAtk * atkScale * 0.05);
            const critDMG = Math.max(finalAtk * atkScale * cDMG - enemy.defense, finalAtk * atkScale * cDMG * 0.05);

            const avgDmg = cRate * critDMG * Math.min(2, this.targets) + (1 - cRate) * hitDmg;
            dps = ((avgDmg / this.attackInterval) * (this.attackSpeed + aspd)) / 100;
        }

        if (this.skillIndex === 1) {
            const skillScale = this.skillParameters[0];
            const finalAtk = this.atk * (1 + this.buffATK) + this.buffATKFlat;
            const hitDmg = Math.max(finalAtk * atkScale - enemy.defense, finalAtk * atkScale * 0.05);
            const critDMG = Math.max(finalAtk * atkScale * cDMG - enemy.defense, finalAtk * atkScale * cDMG * 0.05);

            const skillHit = Math.max(finalAtk * atkScale * skillScale - enemy.defense, finalAtk * atkScale * skillScale * 0.05);
            const skilLCrit = Math.max(finalAtk * atkScale * skillScale * cDMG - enemy.defense, finalAtk * atkScale * skillScale * cDMG * 0.05);

            const avgDmg = cRate * critDMG * Math.min(2, this.targets) + (1 - cRate) * hitDmg;
            const avgSkill = (cRate * skilLCrit * Math.min(2, this.targets) + (1 - cRate) * skillHit) * 2;

            const spCost = this.skillCost / (1 + this.spBoost) + 1.2; // SP lockout
            const atkCycle = this.attackInterval / (this.attackSpeed / 100);
            const atksPerSkillActivation = spCost / atkCycle;

            let avgHit = avgSkill;

            if (atksPerSkillActivation > 1) {
                if (this.skillParameters[1] > 1) {
                    avgHit = (avgSkill + (atksPerSkillActivation - 1) * avgDmg) / atksPerSkillActivation;
                } else {
                    avgHit = (avgSkill + Math.round(atksPerSkillActivation) * avgDmg) / (Math.round(atksPerSkillActivation) + 1);
                }
            }

            dps = ((avgHit / this.attackInterval) * this.attackSpeed) / 100;
        }

        if (this.skillIndex === 2) {
            const atkBuff = this.skillParameters[0];

            this.attackInterval = 1.7;

            const finalAtk = this.atk * (1 + atkBuff + this.buffATK) + this.buffATKFlat;
            const hitDmg = Math.max(finalAtk * atkScale - enemy.defense, finalAtk * atkScale * 0.05);
            const critDMG = Math.max(finalAtk * atkScale * cDMG - enemy.defense, finalAtk * atkScale * cDMG * 0.05);

            const avgDmg = cRate * critDMG * Math.min(2, this.targets) + (1 - cRate) * hitDmg;
            dps = (((3 * avgDmg) / this.attackInterval) * this.attackSpeed) / 100;
        }

        return dps;
    }
}
