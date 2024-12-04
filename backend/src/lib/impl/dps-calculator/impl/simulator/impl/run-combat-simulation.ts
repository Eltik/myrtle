import { AttackType } from "../../../../../../types/impl/lib/impl/dps-calculator";
import OperatorUnit from "../../classes/operator-unit";
import Stats from "../../classes/stats";
import formula from "../../formula";
import SimulationData from "./simulation-data";

export const runCombatSimulation = (operatorUnit: OperatorUnit, skillId: string, enemyUnit: Stats) => {
    const simulationData = new SimulationData();
    const operatorDamageTotalData: number[] = [];
    const operatorDPSTotalData: number[] = [];

    const skill = operatorUnit.skills.find((skill) => skill.skillData.skillId === skillId);
    if (!skill) throw new Error(`Skill ${skillId} not found for operator ${operatorUnit.operator.name}`);

    let damageData = 0;
    let dpsData = 0;

    const operatorAPS = formula.attacksPerSecond(operatorUnit.stats, skill.attackSpeedModifiers, skill.attackIntervalModifiers);
    const operatorFinalAttack = formula.finalAttack(operatorUnit.stats, skill.baseAttackModifiers, skill.attackMultiplierModifiers, 0);

    switch (operatorUnit.stats.attackType) {
        case AttackType.physical:
            damageData = formula.physicalDamage(operatorFinalAttack, enemyUnit, skill.flatDefModifiers, skill.scalingDefModifiers, skill.physTakenModifiers);
            dpsData = formula.damagePerSecond(operatorAPS, damageData);
            break;
        case AttackType.arts:
            damageData = formula.artsDamage(operatorFinalAttack, enemyUnit, skill.flatResModifiers, skill.scalingResModifiers, skill.artsTakenModifiers);
            dpsData = formula.damagePerSecond(operatorAPS, damageData);
            break;
        case AttackType.healing:
            damageData = operatorFinalAttack;
            dpsData = formula.damagePerSecond(operatorAPS, operatorFinalAttack);
            break;
    }

    if (skill.extraPhysDamageDone.length > 0) {
        damageData += formula.physicalDamage(skill.extraPhysDamageDone[0], enemyUnit, skill.flatDefModifiers, skill.scalingDefModifiers, skill.physTakenModifiers);
        dpsData = formula.damagePerSecond(operatorAPS, damageData);
    } else if (skill.extraArtsDamageDone.length > 0) {
        damageData += formula.artsDamage(skill.extraArtsDamageDone[0], enemyUnit, skill.flatResModifiers, skill.scalingResModifiers, skill.artsTakenModifiers);
        dpsData = formula.damagePerSecond(operatorAPS, damageData);
    } else if (skill.extraTrueDamageDone.length > 0) {
        damageData += skill.extraTrueDamageDone[0];
        dpsData = formula.damagePerSecond(operatorAPS, damageData);
    }

    operatorDamageTotalData.push(damageData);
    operatorDPSTotalData.push(dpsData);

    simulationData.operatorDamageTotalData = operatorDamageTotalData;
    simulationData.operatorDPSTotalData = operatorDPSTotalData;
    simulationData.enemyDefData = enemyUnit.def;
    simulationData.enemyResData = enemyUnit.res;

    simulationData.seriesName = "" + operatorUnit.stats.name + " vs " + enemyUnit.name;

    return simulationData;
};
