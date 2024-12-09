import type { OperatorParams } from "../../../../../../types/impl/lib/impl/dps-calculator";
import OperatorUnit from "../../classes";
import { OperatorData } from "../../classes/impl/operator-data";

export default class Archetto extends OperatorUnit {
    constructor(operatorData: OperatorData, params: OperatorParams) {
        super(operatorData, params, 2, 1, 1);

        if (this.operatorModule?.id === "uniequip_003_archet" && this.operatorModuleLevel > 1 && this.talentDamage && this.skillIndex !== 2) {
            if (this.talentDamageName) {
                this.talentDamageName += " +2ndSniper";
            } else {
                this.talentDamageName = "+2ndSniper";
            }
        }

        if (this.moduleDamage && this.operatorModule?.id === "uniequip_003_archet") {
            if (this.moduleDamageName) {
                this.moduleDamageName += " aerialTarget";
            } else {
                this.moduleDamageName = "aerialTarget";
            }
        }

        if (this.moduleDamage && this.operatorModule?.id === "uniequip_002_archet") {
            if (this.moduleDamageName) {
                this.moduleDamageName += " GroundEnemy";
            } else {
                this.moduleDamageName = "GroundEnemy";
            }
        }
    }

    public skillDPS(enemy: { defense: number; res: number }): number {
        const aspd = this.operatorModule?.id === "uniequip_002_archet" && this.moduleDamage ? 8 : 0;
        const atkScale = this.operatorModule?.id === "uniequip_003_archet" && this.moduleDamage ? 1.1 : 1;
        let recoveryInterval = this.elite > 0 ? Math.max(...this.talent1Parameters) : 10000000;
        if (this.operatorModule?.id === "uniequip_003_archet" && this.talentDamage && this.operatorModuleLevel > 1) {
            recoveryInterval -= this.operatorModuleLevel === 2 ? 0.3 : 0.4;
        }

        let dps = 0;
        if (this.skillIndex === 0) {
            const skillScale = this.skillParameters[0];
            const skillScale2 = this.skillParameters[1];
            const spCost = this.skillCost;

            const finalAtk = this.atk * (1 + this.buffATK) + this.buffATKFlat;
            const hitDmg = Math.max(finalAtk * atkScale - enemy.defense, finalAtk * atkScale * 0.05);
            const skillDmg = Math.max(finalAtk * skillScale * atkScale - enemy.defense, finalAtk * skillScale * atkScale * 0.05);
            const aoeDmg = Math.max(finalAtk * skillScale2 * atkScale - enemy.defense, finalAtk * skillScale2 * atkScale * 0.05);

            // Oh boy
            const baseCycleTime = (spCost + 1) / ((this.attackSpeed + aspd) / 100);
            let talentsPerBaseCycle = baseCycleTime / recoveryInterval;
            const failureRate = 1.8 / (spCost + 1);

            talentsPerBaseCycle *= 1 - failureRate;

            const newSpCost = Math.max(1, spCost - talentsPerBaseCycle);

            const hitDPS = ((hitDmg / (this.attackInterval / ((this.attackSpeed + aspd) / 100))) * (newSpCost - 1)) / newSpCost;
            const skillDPS = skillDmg / (this.attackInterval / ((this.attackSpeed + aspd) / 100)) / newSpCost;
            const aoeDPS = (aoeDmg / (this.attackInterval / ((this.attackSpeed + aspd) / 100)) / newSpCost) * (Math.min(this.targets, 4) - 1);

            dps = hitDPS + skillDPS + aoeDPS;
        }

        if (this.skillIndex === 1) {
            const spRecovery = 1 / recoveryInterval + (this.attackSpeed + aspd) / 100;
            const skillScale = this.skillParameters[0];

            const finalAtk = this.atk * (1 + this.buffATK) + this.buffATKFlat;
            const hitDmg = Math.max(finalAtk * atkScale - enemy.defense, finalAtk * atkScale * 0.05);
            const skillDmg = Math.max(finalAtk * skillScale * atkScale - enemy.defense, finalAtk * skillScale * atkScale * 0.05);
            const targets = Math.min(5, this.targets);
            const totalHits = [5, 9, 12, 14, 15];

            dps = hitDmg / (this.attackInterval / ((this.attackSpeed + aspd) / 100)) + (spRecovery / this.skillCost) * skillDmg * totalHits[targets - 1];
        }

        if (this.skillIndex === 0 || this.skillIndex === 2) {
            const finalAtk = this.atk * (1 + this.buffATK + (this.skillParameters[0] * (this.skillIndex + 1)) / 3) + this.buffATKFlat;
            const hitDmg = Math.max(finalAtk * atkScale - enemy.defense, finalAtk * atkScale * 0.05) * (1 + ((this.skillIndex + 1) * 2) / 3);

            dps = ((hitDmg / this.attackInterval) * (this.attackSpeed + aspd)) / 100;

            if (this.skillIndex === 2) {
                dps *= Math.min(this.targets, 2);
            }
        }

        return dps;
    }
}
