import type { OperatorParams } from "../../../../../../../types/impl/lib/impl/dps-calculator";
import OperatorUnit from "../../../classes";
import { OperatorData } from "../../../classes/impl/operator-data";

export default class Broca extends OperatorUnit {
    constructor(operatorData: OperatorData, params: OperatorParams) {
        super(operatorData, params, 1, 1, 1, [0, 1]);

        this.talentDamageNames = ["blocking2+"];
        if (this.talentDamage && this.elite > 0) {
            if (this.talentDamageName) {
                this.talentDamageName += " blocking2+";
            } else {
                this.talentDamageName = "blocking2+";
            }
        }

        this.moduleDamageNames = ["vsBlocked"];
        if (this.operatorModule?.id === "uniequip_002_broca" && this.moduleDamage) {
            if (this.moduleDamageName) {
                this.moduleDamageName += " vsBlocked";
            } else {
                this.moduleDamageName = "vsBlocked";
            }
        }
    }

    public skillDPS(enemy: { defense: number; res: number }): number {
        const atkBuff = (this.talentDamage ? this.talent1Parameters[0] : 0) + (this.skillIndex > -1 ? this.skillParameters[0] : 0);
        const atkScale = this.operatorModule?.id === "uniequip_002_broca" && this.moduleDamage ? 1.1 : 1;

        const finalAtk = this.atk * (1 + this.buffATK + atkBuff) + this.buffATKFlat;
        const attackInterval = this.skillIndex === 1 ? 1.98 : this.attackInterval;
        const hitDmg = this.skillIndex > -1 ? Math.max(finalAtk * atkScale * (1 - enemy.res / 100), finalAtk * atkScale * 0.05) : Math.max(finalAtk * atkScale - enemy.defense, finalAtk * atkScale * 0.05);

        const dps = (((hitDmg / attackInterval) * this.attackSpeed) / 100) * Math.min(3, this.targets);
        return dps;
    }
}
