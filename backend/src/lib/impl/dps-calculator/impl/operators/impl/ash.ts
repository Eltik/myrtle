import type { OperatorParams } from "../../../../../../types/impl/lib/impl/dps-calculator";
import OperatorUnit from "../../classes";
import { OperatorData } from "../../classes/impl/operator-data";

export default class Ash extends OperatorUnit {
    constructor(operatorData: OperatorData, params: OperatorParams) {
        super(operatorData, params, 1, 1, 1);

        this.skillDamageNames = ["vsStunned"];
        if (this.skillDamage && this.skillIndex === 1) {
            this.skillDamageName = "vsStunned";
        }

        this.moduleDamageNames = ["aerialTarget", "groundEnemy"];
        if (this.moduleDamage) {
            if (this.operatorModule?.id === "uniequip_002_ash") {
                this.moduleDamageName = "aerialTarget";
            }

            if (this.operatorModule?.id === "uniequip_003_ash") {
                this.moduleDamageName = "groundEnemy";
            }
        }
    }

    public skillDPS(enemy: { defense: number; res: number }): number {
        let dps = 0;

        let atkScale = this.operatorModule?.id === "uniequip_002_ash" && this.moduleDamage ? 1.1 : 1;
        const aspd = this.operatorModule?.id === "uniequip_003_ash" && this.moduleDamage ? 8 : 0;

        if (this.skillIndex < 1) {
            const finalAtk = this.atk * (1 + this.buffATK + this.skillParameters[0] * (this.skillIndex + 1)) + this.buffATKFlat;
            const hitDmg = Math.max(finalAtk * atkScale - enemy.defense, finalAtk * atkScale * 0.05);

            dps = (((hitDmg / this.attackInterval) * (this.attackSpeed + aspd)) / 100) * (1 + (this.skillIndex + 1));
        }

        if (this.skillIndex === 1) {
            const attackInterval = 0.2;

            const finalAtk = this.atk * (1 + this.buffATK) + this.buffATKFlat;

            if (this.skillDamage) {
                atkScale *= this.skillParameters[1];
            }

            const hitDmg = Math.max(finalAtk * atkScale - enemy.defense, finalAtk * atkScale * 0.05);
            const dmgBonus = this.operatorModule?.id === "uniequip_002_ash" && this.operatorModuleLevel > 1 && this.skillDamage ? this.talent1Parameters[2] : 1;

            dps = (((hitDmg / attackInterval) * (this.attackSpeed + aspd)) / 100) * dmgBonus;
        }

        return dps;
    }
}
