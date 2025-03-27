import type { OperatorParams } from "../../../../../../../types/impl/lib/impl/dps-calculator";
import OperatorUnit from "../../../classes";
import { OperatorData } from "../../../classes/impl/operator-data";

export default class Bibeak extends OperatorUnit {
    constructor(operatorData: OperatorData, params: OperatorParams) {
        super(operatorData, params, 1, 6, 1, [0, 1]);

        this.talentDamageNames = ["+ASPD"];
        if (this.elite > 0 && this.elite > 0) {
            if (this.talentDamageName) {
                this.talentDamageName += ` +ASPD`;
            } else {
                this.talentDamageName = `+ASPD`;
            }
        }
    }

    public skillDPS(enemy: { defense: number; res: number }): number {
        let dps = 0;

        const aspd = this.talentDamage ? this.talent1Parameters[0] * this.talent1Parameters[1] : 0;
        const atkBuff = this.talentDamage && this.operatorModule?.id === "uniequip_002_bibeak" && this.operatorModuleLevel > 1 ? 0.01 * (this.operatorModuleLevel - 1) * this.talent1Parameters[1] : 0;
        const dmgMultiplier = this.operatorModule?.id === "uniequip_002_bibeak" ? 1.1 : 1;
        const finalAtk = this.atk * (1 + atkBuff + this.buffATK) + this.buffATKFlat;
        const hitDmg = Math.max(finalAtk - enemy.defense, finalAtk * 0.05);

        if (this.skillIndex < 1) {
            const skillScale = this.skillParameters[0];
            let skillHitDmg = Math.max(finalAtk * skillScale - enemy.defense, finalAtk * skillScale * 0.05) * dmgMultiplier;
            const skillArtsDmg = Math.max(finalAtk * skillScale * (1 - enemy.res / 100), finalAtk * skillScale * 0.05) * dmgMultiplier;

            if (this.skillIndex === -1) skillHitDmg = hitDmg;

            const spCost = this.skillCost;
            const avgPhysical = (2 * (spCost * hitDmg + skillHitDmg)) / (spCost + 1);
            const avgArts = this.targets === 1 ? 0 : (skillArtsDmg / (spCost + 1)) * (this.skillIndex + 1);

            dps = (((avgPhysical + avgArts) / this.attackInterval) * (this.attackSpeed + aspd)) / 100;
        }

        if (this.skillIndex === 1) {
            const skillScale = this.skillParameters[2];
            const skillArtsDmg = Math.max(finalAtk * skillScale * (1 - enemy.res / 100), finalAtk * skillScale * 0.05) * dmgMultiplier;
            const avgHit = (2 * hitDmg * this.skillCost + skillArtsDmg * Math.min(this.targets, this.skillParameters[0])) / this.skillCost;
            dps = ((avgHit / this.attackInterval) * (this.attackSpeed + aspd)) / 100;
        }

        return dps;
    }
}
