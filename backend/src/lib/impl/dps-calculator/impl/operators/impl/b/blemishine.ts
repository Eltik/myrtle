import type { OperatorParams } from "../../../../../../../types/impl/lib/impl/dps-calculator";
import OperatorUnit from "../../../classes";
import { OperatorData } from "../../../classes/impl/operator-data";

export default class Blemishine extends OperatorUnit {
    constructor(operatorData: OperatorData, params: OperatorParams) {
        super(operatorData, params, 2, 1, 2, [0, 1, 2]);

        this.talent2DamageNames = ["vsSleep", "w/o sleep"];
        if (this.elite > 0) {
            if (this.talent2Damage) {
                if (this.talent2DamageName) {
                    this.talent2DamageName += " vsSleep";
                } else {
                    this.talent2DamageName = "vsSleep";
                }
            } else {
                if (this.talent2DamageName) {
                    this.talent2DamageName += " w/o sleep";
                } else {
                    this.talent2DamageName = "w/o sleep";
                }
            }
        }
    }

    public skillDPS(enemy: { defense: number; res: number }): number {
        let dps = 0;
        const atkScale = this.talent2Damage ? this.talent2Parameters[0] : 1;

        if (this.skillIndex < 1) {
            const skillScale = this.skillParameters[0];
            const finalAtk = this.atk * (1 + this.buffATK) + this.buffATKFlat;

            const hitDmg = Math.max(finalAtk * atkScale - enemy.defense, finalAtk * atkScale * 0.05);
            const skillDmg = this.skillIndex === -1 ? hitDmg : Math.max(finalAtk * atkScale * skillScale - enemy.defense, finalAtk * atkScale * skillScale * 0.05);

            const spCost = this.skillCost / (1 + this.spBoost) + 1.2; // SP lockout
            const atkCycle = this.attackInterval / (this.attackSpeed / 100);
            const atksPerSkillActivation = spCost / atkCycle;

            let avgHit = skillDmg;
            if (atksPerSkillActivation > 1) {
                if (this.skillParameters[2] > 1) {
                    avgHit = (skillDmg + (atksPerSkillActivation - 1) * hitDmg) / atksPerSkillActivation;
                } else {
                    avgHit = (skillDmg + Math.floor(atksPerSkillActivation) * hitDmg) / (Math.floor(atksPerSkillActivation) + 1);
                }
            }

            dps = ((avgHit / this.attackInterval) * this.attackSpeed) / 100;
        }

        if (this.skillIndex === 1) {
            const atkBuff = this.skillParameters[0];
            const finalAtk = this.atk * (1 + atkBuff + this.buffATK) + this.buffATKFlat;

            const hitDmg = Math.max(finalAtk * atkScale - enemy.defense, finalAtk * atkScale * 0.05);
            dps = ((hitDmg / this.attackInterval) * this.attackSpeed) / 100;
        }

        if (this.skillIndex === 2) {
            const atkBuff = this.skillParameters[0];
            const finalAtk = this.atk * (1 + atkBuff + this.buffATK) + this.buffATKFlat;

            const hitDmg = Math.max(finalAtk * atkScale - enemy.defense, finalAtk * atkScale * 0.05);
            const artsDamage = Math.max(finalAtk * atkScale * this.skillParameters[2] * (1 - enemy.res / 100), finalAtk * atkScale * this.skillParameters[2] * 0.05);
            dps = (((hitDmg + artsDamage) / this.attackInterval) * this.attackSpeed) / 100;
        }

        return dps;
    }
}
