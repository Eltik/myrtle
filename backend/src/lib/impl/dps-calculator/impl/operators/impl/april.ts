import type { OperatorParams } from "../../../../../../types/impl/lib/impl/dps-calculator";
import OperatorUnit from "../../classes";
import { OperatorData } from "../../classes/impl/operator-data";

export default class April extends OperatorUnit {
    constructor(operatorData: OperatorData, params: OperatorParams) {
        super(operatorData, params, 1, 1, 1);

        if (this.moduleDamage && this.operatorModule?.id === "uniequip_002_aprl") {
            if (this.moduleDamageName) {
                this.moduleDamageName += " groundEnemies";
            } else {
                this.moduleDamageName = "groundEnemies";
            }
        }
    }

    public skillDPS(enemy: { defense: number; res: number }): number {
        const aspd = this.operatorModule?.id === "uniequip_002_aprl" ? 8 : 0;
        let dps = 0;

        if (this.skillIndex === 0) {
            const finalAtk = this.atk * (1 + this.buffATK) + this.buffATKFlat;
            const hitDmg = Math.max(finalAtk - enemy.defense, finalAtk * 0.05);
            const skillDmg = Math.max(finalAtk * this.skillParameters[0] - enemy.defense, finalAtk * this.skillParameters[0] * 0.05);
            const avgDmg = (this.skillCost * hitDmg + skillDmg) / (this.skillCost + 1);

            dps = ((avgDmg / this.attackInterval) * (this.attackSpeed + aspd)) / 100;
        }

        if (this.skillIndex === 1) {
            const finalAtk = this.atk * (1 + this.buffATK + (this.skillParameters[0] * (this.skillIndex + 1)) / 2) + this.buffATKFlat;
            const hitDmg = Math.max(finalAtk - enemy.defense, finalAtk * 0.05);

            dps = ((hitDmg / this.attackInterval) * (this.attackSpeed + aspd)) / 100;
        }
        return dps;
    }
}
