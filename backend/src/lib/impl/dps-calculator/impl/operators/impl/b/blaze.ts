import type { OperatorParams } from "../../../../../../../types/impl/lib/impl/dps-calculator";
import OperatorUnit from "../../../classes";
import { OperatorData } from "../../../classes/impl/operator-data";

export default class Blaze extends OperatorUnit {
    constructor(operatorData: OperatorData, params: OperatorParams) {
        super(operatorData, params, 1, 1, 1, [0, 1]);

        this.moduleDamageNames = ["vsBlocked"];
        if (this.operatorModule?.id === "uniequip_002_huang" && this.moduleDamage) {
            if (this.moduleDamageName) {
                this.moduleDamageName += ` vsBlocked`;
            } else {
                this.moduleDamageName = `vsBlocked`;
            }
        }
    }

    public skillDPS(enemy: { defense: number; res: number }): number {
        let dps = 0;

        let atkBuff = 0;
        let aspd = 0;
        const atkScale = this.moduleDamage && this.operatorModule?.id === "uniequip_002_huang" ? 1.1 : 1;

        const targets = this.elite === 2 ? 3 : 2;

        if ((this.talent2Damage || this.skillIndex === 1) && this.operatorModule?.id === "uniequip_002_huang" && this.operatorModuleLevel > 1) {
            atkBuff = this.talent2Parameters[0];
            aspd = this.talent2Parameters[1];
        }

        if (this.skillIndex === 0) {
            const skillScale = this.skillParameters[0];
            const finalAtk = this.atk * (1 + atkBuff + this.buffATK) + this.buffATKFlat;

            const hitDmg = Math.max(finalAtk * atkScale - enemy.defense, finalAtk * atkScale * 0.05);
            const skillHitDmg = Math.max(finalAtk * atkScale * skillScale - enemy.defense, finalAtk * atkScale * skillScale * 0.05);

            const spCost = this.skillCost;
            const avgPhysical = ((spCost * hitDmg + skillHitDmg) / (spCost + 1)) * Math.min(this.targets, targets);
            dps = ((avgPhysical / this.attackInterval) * (this.attackSpeed + aspd)) / 100;
        } else {
            atkBuff += (this.skillParameters[0] * (this.skillIndex + 1)) / 2;

            const finalAtk = this.atk * (1 + atkBuff + this.buffATK) + this.buffATKFlat;
            const hitDmg = Math.max(finalAtk * atkScale - enemy.defense, finalAtk * atkScale * 0.05) * Math.min(this.targets, targets);
            dps = ((hitDmg / this.attackInterval) * (this.attackSpeed + aspd)) / 100;
        }

        return dps;
    }
}
