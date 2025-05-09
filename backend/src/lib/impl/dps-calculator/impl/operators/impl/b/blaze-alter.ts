/**
 * @todo: Need to test. Blaze Alter is not in the game yet.
 */

import type { OperatorParams } from "../../../../../../../types/impl/lib/impl/dps-calculator";
import OperatorUnit from "../../../classes";
import { OperatorData } from "../../../classes/impl/operator-data";

export default class BlazeAlter extends OperatorUnit {
    constructor(operatorData: OperatorData, params: OperatorParams) {
        super(operatorData, params, 2, 1, 0, [0, 1, 2]);

        this.traitDamageNames = ["burning"];
        this.skillDamageNames = ["avgBurn", "avgBurn vsBoss", "vsBurn"];
        if (this.skillIndex === 0 || this.skillIndex === 1) {
            if (this.traitDamage) {
                if (this.traitDamageName) {
                    this.traitDamageName += " burning";
                } else {
                    this.traitDamageName = "burning";
                }
            } else {
                if (this.skillDamage) {
                    if (this.skillDamageName) {
                        this.skillDamageName += " avgBurn";
                    } else {
                        this.skillDamageName = "avgBurn";
                    }
                } else {
                    if (this.skillDamageName) {
                        this.skillDamageName += " avgBurn vsBoss";
                    } else {
                        this.skillDamageName = "avgBurn vsBoss";
                    }
                }
            }
        }

        if (this.skillIndex === 2 && this.skillDamage) {
            if (this.skillDamageName) {
                this.skillDamageName += " vsBurn";
            } else {
                this.skillDamageName = "vsBurn";
            }
        }
    }

    public skillDPS(enemy: { defense: number; res: number }): number {
        let dps = 0;

        const atkBuff = this.skillIndex === 1 ? this.skillParameters[0] : 0;
        const finalAtk = this.atk * (1 + this.buffATK + atkBuff) + this.buffATKFlat;

        const FALLOUT_DAMAGE = this.elite > 0 ? 7000 + finalAtk * this.talent1Parameters[0] : 7000;

        if (this.skillIndex === -1) {
            const finalAtk = this.atk * (1 + this.buffATK) + this.buffATKFlat;
            const hitDmg = Math.max(finalAtk * (1 - enemy.res / 100), finalAtk * 0.05);
            dps = ((hitDmg / this.attackInterval) * this.attackSpeed) / 100;
        }

        if (this.skillIndex === 0) {
            const skillScale = this.skillParameters[0];
            const finalAtk = this.atk * (1 + this.buffATK) + this.buffATKFlat;

            const newRes = Math.max(0, enemy.res - 20);
            const elementalGuage = this.skillDamage ? 1000 : 2000;

            const hitDmg1 = Math.max(finalAtk * (1 - enemy.res / 100), finalAtk * 0.05);
            const hitDmg2 = Math.max(finalAtk * (1 - newRes / 100), finalAtk * 0.05); // Against enemies under burn fallout

            const skillDmg1 = Math.max(finalAtk * skillScale * (1 - enemy.res / 100), finalAtk * skillScale * 0.05);
            const skillDmg2 = Math.max(finalAtk * skillScale * (1 - newRes / 100), finalAtk * skillScale * 0.05); // Against enemies under burn fallout

            const dpsNorm = ((hitDmg1 / this.attackInterval) * this.attackSpeed) / 100 + skillDmg1 * this.targets;
            const dpsFallout = ((hitDmg2 / this.attackInterval) * this.attackSpeed) / 100 + skillDmg2 * this.targets;

            const timeToFallout = elementalGuage / (skillDmg1 * this.skillParameters[1]);

            dps = (dpsNorm * timeToFallout + dpsFallout * 10 + FALLOUT_DAMAGE) / (timeToFallout + 10);
            if (!this.traitDamage) dps = dpsNorm;
        }

        if (this.skillIndex === 1) {
            const atkBuff = this.skillParameters[0];
            const skillScale = this.skillParameters[2];
            const finalAtk = this.atk * (1 + this.buffATK + atkBuff) + this.buffATKFlat;

            const newRes = Math.max(0, enemy.res - 20);
            const elementalGuage = this.skillDamage ? 1000 : 2000;

            const hitDmg1 = Math.max(finalAtk * (1 - enemy.res / 100), finalAtk * 0.05) * Math.min(this.targets, 3);
            const hitDmg2 = Math.max(finalAtk * (1 - newRes / 100), finalAtk * 0.05) * Math.min(this.targets, 3); // Against enemies under burn fallout

            const skillDmg1 = Math.max(finalAtk * skillScale * (1 - enemy.res / 100), finalAtk * skillScale * 0.05);
            const skillDmg2 = Math.max(finalAtk * skillScale * (1 - newRes / 100), finalAtk * skillScale * 0.05); // Against enemies under burn fallout

            const dpsNorm = ((hitDmg1 / 2.5) * this.attackSpeed) / 100 + skillDmg1 * this.targets;
            const dpsFallout = ((hitDmg2 / 2.5) * this.attackSpeed) / 100 + skillDmg2 * this.targets;

            const timeToFallout = elementalGuage / (skillDmg1 * this.skillParameters[1]);

            dps = (dpsNorm * timeToFallout + dpsFallout * 10 + FALLOUT_DAMAGE) / (timeToFallout + 10);
            if (!this.traitDamage) dps = dpsNorm;
        }

        if (this.skillIndex === 2) {
            const atkBuff = this.skillParameters[0];
            const finalAtk = this.atk * (1 + this.buffATK + atkBuff) + this.buffATKFlat;

            const newRes = this.skillDamage ? Math.max(0, enemy.res - 20) : enemy.res;
            const elementalScale = this.skillDamage ? this.skillParameters[3] : 0;

            const hitDmg = Math.max(finalAtk * (1 - newRes / 100), finalAtk * 0.05) + finalAtk * elementalScale;
            dps = (((hitDmg / 0.3) * this.attackSpeed) / 100) * this.targets;
        }

        return dps;
    }
}
