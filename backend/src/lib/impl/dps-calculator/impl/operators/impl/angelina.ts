import type { OperatorParams } from "../../../../../../types/impl/lib/impl/dps-calculator";
import OperatorUnit from "../../classes";
import { OperatorData } from "../../classes/impl/operator-data";

export default class Angelina extends OperatorUnit {
    constructor(operatorData: OperatorData, params: OperatorParams) {
        super(operatorData, params, 2, 1, 1);
    }

    public skillDPS(enemy: { defense: number; res: number }): number {
        let aspd = this.talent1Parameters[0];

        if (this.operatorModule?.id === "uniequip_003_aglina") {
            if (this.operatorModuleLevel === 2) aspd += 3;
            if (this.operatorModuleLevel === 3) aspd += 5;
        }

        let dps = 0;
        if (this.skillIndex === 0) {
            const finalAtk = this.atk * (1 + this.buffATK + this.skillParameters[0] * (this.skillIndex + 1)) + this.buffATKFlat;
            const hitDmg = Math.max(finalAtk * (1 - enemy.res / 100), finalAtk * 0.05);

            dps = ((hitDmg / this.attackInterval) * (this.attackSpeed + aspd)) / 100;
        }

        if (this.skillIndex === 1) {
            const attackInterval = this.attackInterval * 0.15;
            const skillScale = this.skillParameters[1];

            const finalAtk = this.atk * (1 + this.buffATK) + this.buffATKFlat;
            const hitDmg = Math.max(finalAtk * skillScale * (1 - enemy.res / 100), finalAtk * skillScale * 0.05);

            dps = ((hitDmg / attackInterval) * (this.attackSpeed + aspd)) / 100;
        }

        if (this.skillIndex === 2) {
            const targets = this.skillParameters[1];

            const finalAtk = this.atk * (1 + this.buffATK + this.skillParameters[0]) + this.buffATKFlat;
            const hitDmg = Math.max(finalAtk * (1 - enemy.res / 100), finalAtk * 0.05);

            dps = (((hitDmg / this.attackInterval) * (this.attackSpeed + aspd)) / 100) * Math.min(this.targets, targets);
        }

        return dps;
    }
}
