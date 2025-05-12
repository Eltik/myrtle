import type { OperatorParams } from "../../../../../../../types/impl/lib/impl/dps-calculator";
import OperatorUnit from "../../../classes";
import { OperatorData } from "../../../classes/impl/operator-data";

export default class Bryophyta extends OperatorUnit {
    constructor(operatorData: OperatorData, params: OperatorParams) {
        super(operatorData, params, 2, 6, 1, [0, 1]);

        this.traitDamageNames = ["blocking"];
        if (!this.traitDamage) {
            if (this.traitDamageName) {
                this.traitDamageName += " blocking";
            } else {
                this.traitDamageName = "blocking";
            }
        }
    }

    public skillDPS(enemy: { defense: number; res: number }): number {
        let dps = 0;
        const atkScale = this.traitDamage ? (this.operatorModule?.id === "uniequip_002_bryota" ? 1.3 : 1.2) : 1;

        if (this.skillIndex === 0) {
            const skillScale = this.skillParameters[0];
            const finalAtk = this.atk * (1 + this.buffATK) + this.buffATKFlat;
            const hitDmg = Math.max(finalAtk * atkScale - enemy.defense, finalAtk * atkScale * 0.05);
            const skillHitDmg = Math.max(finalAtk * atkScale * skillScale - enemy.defense, finalAtk * atkScale * skillScale * 0.05);
            const spCost = this.skillCost;

            const avgPhysical = (spCost * hitDmg + skillHitDmg) / (spCost + 1);
            dps = (avgPhysical / this.attackInterval) * (this.attackSpeed / 100);
        } else {
            const finalAtk = this.atk * (1 + this.buffATK + (this.skillParameters[0] * (this.skillIndex + 1)) / 2) + this.buffATKFlat;
            const hitDmg = Math.max(finalAtk * atkScale - enemy.defense, finalAtk * atkScale * 0.05);
            dps = (hitDmg / this.attackInterval) * (this.attackSpeed / 100);
        }

        return dps;
    }
}
