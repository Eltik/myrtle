import type { OperatorParams } from "../../../../../../../types/impl/lib/impl/dps-calculator";
import OperatorUnit from "../../../classes";
import { OperatorData } from "../../../classes/impl/operator-data";

export default class Aak extends OperatorUnit {
    constructor(operatorData: OperatorData, params: OperatorParams) {
        super(operatorData, params, 2, 1, 1, [0, 2]);
    }

    public skillDPS(enemy: { defense: number; res: number }): number {
        const cdmg = Math.max(...this.talent1Parameters);
        let critRate = 0.25;

        if (this.operatorModule && this.operatorModuleLevel > 1) {
            if (this.operatorModuleLevel === 2) {
                critRate = 0.25 + 0.75 * 0.2;
            } else {
                critRate = 0.25 + 0.75 * 0.3;
            }
        }

        const finalAtk = this.skillIndex === 2 ? this.atk * (1 + this.buffATK + this.skillParameters[0] + this.buffATKFlat) : this.atk * (1 + this.buffATK) + this.buffATKFlat;
        const aspd = this.skillIndex === 0 ? this.skillParameters[0] : (this.skillParameters[1] * (this.skillIndex + 1)) / 3;
        const hitDmg = Math.max(finalAtk - enemy.defense, finalAtk * 0.05);
        const critDMG = Math.max(finalAtk * cdmg - enemy.defense, finalAtk * cdmg * 0.05);
        const avgHit = (1 - critRate) * hitDmg + critRate * critDMG;
        const dps = ((avgHit / this.attackInterval) * (this.attackSpeed + aspd)) / 100;
        return dps;
    }
}
