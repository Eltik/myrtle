import type { OperatorParams } from "../../../../../../types/impl/lib/impl/dps-calculator";
import OperatorUnit from "../../classes";
import { OperatorData } from "../../classes/impl/operator-data";

export default class Ashlock extends OperatorUnit {
    constructor(operatorData: OperatorData, params: OperatorParams) {
        super(operatorData, params, 1, 1, 1, [0, 1]);

        this.talentDamageNames = ["LowTalent"];
        if (!this.talentDamage) {
            if (this.talentDamageName) {
                this.talentDamageName += " LowTalent";
            } else {
                this.talentDamageName = "LowTalent";
            }
        }

        this.moduleDamageNames = ["blockedTarget"];
        if (this.moduleDamage && this.operatorModule?.id === "uniequip_002_ashlok") {
            if (this.moduleDamageName) {
                this.moduleDamageName += " blockedTarget";
            } else {
                this.moduleDamageName = "blockedTarget";
            }
        }
    }

    public skillDPS(enemy: { defense: number; res: number }): number {
        let atkBuff = this.talentDamage ? this.talent1Parameters[1] : this.talent1Parameters[0];
        const atkScale = this.operatorModule?.id === "uniequip_002_ashlok" ? 1.1 : 1;
        atkBuff = this.skillIndex > -1 ? this.skillParameters[0] : 0;

        const finalAtk = this.atk * (1 + atkBuff + this.buffATK + atkBuff) + this.buffATKFlat;
        const hitDmg = Math.max(finalAtk * atkScale - enemy.defense, finalAtk * atkScale * 0.05);

        const atkInterval = this.skillIndex !== 1 ? this.attackInterval : this.attackInterval * (1 + this.skillParameters[1]);

        const dps = (((hitDmg / atkInterval) * this.attackSpeed) / 100) * this.targets;
        return dps;
    }
}
