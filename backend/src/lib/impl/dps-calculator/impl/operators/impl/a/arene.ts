import type { OperatorParams } from "../../../../../../../types/impl/lib/impl/dps-calculator";
import OperatorUnit from "../../../classes";
import { OperatorData } from "../../../classes/impl/operator-data";

export default class Arene extends OperatorUnit {
    constructor(operatorData: OperatorData, params: OperatorParams) {
        super(operatorData, params, 1, 6, 1, [0, 1]);

        if (this.skillIndex === 0 && this.talentDamage) {
            this.traitDamage = false;
        }

        this.traitDamageNames = ["rangedAtk"];
        if (!this.traitDamage && this.skillIndex === 0) {
            if (this.traitDamageName) {
                this.traitDamageName += " rangedAtk";
            } else {
                this.traitDamageName = "rangedAtk";
            }
        }

        this.talentDamageNames = ["vsDrones"];
        if (this.talentDamage && this.elite > 0) {
            if (this.talentDamageName) {
                this.talentDamageName += " vsDrones";
            } else {
                this.talentDamageName = "vsDrones";
            }
        }

        this.moduleDamageNames = ["+12aspd(mod)"];
        if (this.operatorModule?.id === "uniequip_002_spikes" && this.targets === 1 && this.moduleDamage) {
            if (this.moduleDamageName) {
                this.moduleDamageName += " +12aspd(mod)";
            } else {
                this.moduleDamageName = "+12aspd(mod)";
            }
        }
    }

    public skillDPS(enemy: { defense: number; res: number }): number {
        let atkScale = this.talentDamage ? this.talent1Parameters[0] : 1;
        if (!this.traitDamage && this.skillIndex !== 1) atkScale *= 0.8;
        const aspd = (this.targets === 1 || this.moduleDamage) && this.operatorModule?.id === "uniequip_002_spikes" ? 12 : 0;
        const skillScale = this.skillParameters[0];

        const finalAtk = this.atk * (1 + this.buffATK) + this.buffATKFlat;
        let dps = 0;

        if (this.skillIndex === -1) {
            const hitDmg = Math.max(finalAtk * atkScale - enemy.defense, finalAtk * atkScale * 0.05);
            dps = ((hitDmg / this.attackInterval) * (this.attackSpeed + aspd)) / 100;
        }
        if (this.skillIndex === 0) {
            const hitDmg = Math.max(finalAtk * atkScale * skillScale - enemy.defense, finalAtk * atkScale * skillScale * 0.05);
            dps = (((2 * hitDmg) / this.attackInterval) * (this.attackSpeed + aspd)) / 100;
        }
        if (this.skillIndex === 1) {
            const hitDmgArts = Math.max(finalAtk * skillScale * atkScale * (1 - enemy.res / 100), finalAtk * skillScale * atkScale * 0.05);
            dps = (((hitDmgArts / this.attackInterval) * (this.attackSpeed + aspd)) / 100) * Math.min(2, this.targets);
        }

        return dps;
    }
}
