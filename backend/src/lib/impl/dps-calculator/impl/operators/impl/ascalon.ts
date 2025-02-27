import type { OperatorParams } from "../../../../../../types/impl/lib/impl/dps-calculator";
import OperatorUnit from "../../classes";
import { OperatorData } from "../../classes/impl/operator-data";

export default class Ascalon extends OperatorUnit {
    constructor(operatorData: OperatorData, params: OperatorParams) {
        super(operatorData, params, 1, 1, 1, [0, 1, 2]);

        this.talentDamageNames = ["1Stack", "3Stacks"];
        if (!this.talentDamage) {
            if (this.talentDamageName) {
                this.talentDamageName += " 1Stack";
            } else {
                this.talentDamageName = "1Stack";
            }
        } else {
            if (this.talentDamageName) {
                this.talentDamageName += " 3Stacks";
            } else {
                this.talentDamageName = "3Stacks";
            }
        }

        this.talentDamageNames = ["NoRangedTiles", "nextToRangedTile"];
        if (this.elite === 2) {
            if (!this.talent2Damage) {
                if (this.talent2DamageName) {
                    this.talent2DamageName += " NoRangedTiles";
                } else {
                    this.talent2DamageName = "NoRangedTiles";
                }
            } else {
                if (this.talent2DamageName) {
                    this.talent2DamageName += " nextToRangedTile";
                } else {
                    this.talent2DamageName = "nextToRangedTile";
                }
            }
        }

        this.skillDamageNames = [`${this.targets}targets`];
        if (this.targets > 1) {
            if (this.skillDamageName) {
                this.skillDamageName += ` ${this.targets}targets`;
            } else {
                this.skillDamageName = `${this.targets}targets`;
            }
        }
    }

    public skillDPS(enemy: { defense: number; res: number }): number {
        let dps = 0;
        let finalAtk = 0;
        const talentStacks = this.talentDamage ? 3 : 1;
        const talentScale = this.talent1Parameters[1];
        let aspd = this.talent2Parameters[0];

        if (this.elite === 2 && this.talent2Damage) {
            aspd += this.talent2Parameters[1];
        }

        if (this.skillIndex === 0) {
            const skillScale = this.skillParameters[0];

            finalAtk = this.atk * (1 + this.buffATK) + this.buffATKFlat;
            const hitDmg = Math.max(finalAtk - enemy.defense, finalAtk * 0.05);
            const skillDmg = Math.max(finalAtk * skillScale - enemy.defense, finalAtk * skillScale * 0.05) * 2;

            const spCost = this.skillCost + 1.2; // SP lockout
            const atkCycle = (this.attackInterval / (this.attackSpeed + aspd)) * 100;

            const atksPerSkillActivation = spCost / atkCycle;
            const averageHit = atksPerSkillActivation > 1 ? (skillDmg + (atksPerSkillActivation - 1) * hitDmg) / atksPerSkillActivation : skillDmg;

            dps = (((averageHit / this.attackInterval) * (this.attackSpeed + aspd)) / 100) * this.targets;
        }
        if (this.skillIndex === -1 || this.skillIndex === 1) {
            finalAtk = this.atk * (1 + this.buffATK + (this.skillParameters[0] * (this.skillIndex + 1)) / 2) + this.buffATKFlat;
            const hitDmg = Math.max(finalAtk - enemy.defense, finalAtk * 0.05);

            dps = (((hitDmg / this.attackInterval) * (this.attackSpeed + aspd)) / 100) * this.targets;
        }

        if (this.skillIndex === 2) {
            const attackInterval = this.attackInterval + this.skillParameters[0];
            finalAtk = this.atk * (1 + this.buffATK + this.skillParameters[1]) + this.buffATKFlat;
            const hitDmg = Math.max(finalAtk - enemy.defense, finalAtk * 0.05);

            dps = (((hitDmg / attackInterval) * (this.attackSpeed + aspd)) / 100) * this.targets;
        }

        dps += this.targets * finalAtk * talentStacks * talentScale * Math.max(1 - enemy.res / 100, 0.05);

        return dps;
    }
}
