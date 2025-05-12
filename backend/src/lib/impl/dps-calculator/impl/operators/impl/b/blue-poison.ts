import type { OperatorParams } from "../../../../../../../types/impl/lib/impl/dps-calculator";
import OperatorUnit from "../../../classes";
import { OperatorData } from "../../../classes/impl/operator-data";

export default class BluePoison extends OperatorUnit {
    constructor(operatorData: OperatorData, params: OperatorParams) {
        super(operatorData, params, 0, 1, 1, [0, 1]);

        this.moduleDamageNames = ["GroundTargets"];
        if (this.operatorModule?.id === "uniequip_002_bluep" && this.moduleDamage) {
            if (this.moduleDamageName) {
                this.moduleDamageName += " vsStun";
            } else {
                this.moduleDamageName = "vsStun";
            }
        }
    }

    public skillDPS(enemy: { defense: number; res: number }): number {
        let dps = 0;

        const aspd = this.operatorModule?.id === "uniequip_002_bluep" && this.moduleDamage ? 8 : 0;
        const artsDMG = this.talent1Parameters[1];
        const artsDPS = this.elite > 0 ? Math.max(artsDMG * (1 - enemy.res / 100), artsDMG * 0.05) : 0;

        if (this.skillIndex < 1) {
            const skillScale = this.skillParameters[0];
            const finalAtk = this.atk * (1 + this.buffATK) + this.buffATKFlat;
            const hitDmg = Math.max(finalAtk - enemy.defense, finalAtk * 0.05);
            const skillHitDmg = this.skillIndex === -1 ? hitDmg : Math.max(finalAtk * skillScale - enemy.defense, finalAtk * skillScale * 0.05) * Math.min(2, this.targets);

            const spCost = this.skillCost;
            const avgPhysical = (spCost * hitDmg + skillHitDmg) / (spCost + 1);

            dps = avgPhysical / (this.attackInterval / ((this.attackSpeed + aspd) / 100)) + artsDPS * Math.min(1 + (this.skillIndex + 1), this.targets);
        }

        if (this.skillIndex === 1) {
            const atkBuff = this.skillParameters[0];
            const finalAtk = this.atk * (1 + this.buffATK + atkBuff) + this.buffATKFlat;
            const hitDmg = Math.max(finalAtk - enemy.defense, finalAtk * 0.05);

            dps = (this.skillParameters[1] * hitDmg) / (this.attackInterval / ((this.attackSpeed + aspd) / 100)) + (hitDmg / (this.attackInterval / ((this.attackSpeed + aspd) / 100))) * Math.min(2, this.targets - 1) + artsDPS * Math.min(3, this.targets);
        }

        return dps;
    }
}
