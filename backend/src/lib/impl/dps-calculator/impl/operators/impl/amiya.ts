import type { OperatorParams } from "../../../../../../types/impl/lib/impl/dps-calculator";
import OperatorUnit from "../../classes";
import { OperatorData } from "../../classes/impl/operator-data";

export default class Amiya extends OperatorUnit {
    private s3: number;

    constructor(operatorData: OperatorData, params: OperatorParams) {
        super(operatorData, params, 2, 6, 1);

        if (this.elite === 2 && (this.skillIndex === 2 || this.skillIndex === -1)) {
            this.skillIndex = 2;

            this.s3 = [0, 1, 1.1, 1.2, 1.3, 1.4, 1.5, 1.6, 1.8, 2, 2.3][this.skillLevel ?? 0];
        } else {
            this.s3 = 0;
        }
    }

    public skillDPS(enemy: { defense: number; res: number }): number {
        let dps = 0;
        if (this.skillIndex < 1) {
            const aspd = this.skillParameters[0] * (this.skillIndex + 1);

            const finalAtk = this.atk * (1 + this.buffATK) + this.buffATKFlat;
            const hitDmgArts = Math.max(finalAtk * (1 - enemy.res / 100), finalAtk * 0.05);

            dps = hitDmgArts / (this.attackInterval / ((this.attackSpeed + aspd) / 100));
        }

        if (this.skillIndex === 1) {
            const atkScale = this.skillParameters[0];
            const hits = this.skillParameters[1];

            const finalAtk = this.atk * (1 + this.buffATK) + this.buffATKFlat;
            const hitDmgArts = Math.max(finalAtk * atkScale * (1 - enemy.res / 100), finalAtk * atkScale * 0.05);

            dps = (hits * hitDmgArts) / (this.attackInterval / (this.attackSpeed / 100));
        }

        if (this.skillIndex === 2) {
            const finalAtk = this.atk * (1 + this.buffATK + this.s3) + this.buffATKFlat;

            dps = (finalAtk / (this.attackInterval / (this.attackSpeed / 100))) * Math.max(1, -enemy.defense);
        }

        return dps;
    }
}
