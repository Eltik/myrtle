import { AttackType } from "../../../../../../types/impl/lib/impl/dps-calculator";
import { BattleEquip } from "../../../../../../types/impl/lib/impl/local/impl/gamedata/impl/modules";
import Stats from "../../classes/stats";
import formula from "../../formula";
import { ALL_OPERATORS } from "../../operators";
import SimulationData from "./simulation-data";

export const runCombatSimulation = (
    operatorData: {
        operatorId: string;
        metadata: {
            phaseIndex: number;
            favorPoint: number;
            potentialRank: number;
            moduleId: string;
            moduleLevel: number;
        };
        level: number;
        moduleData?: BattleEquip;
    },
    skillData: {
        skillId: string;
        levelIndex: number;
    },
    enemyUnit: Stats,
) => {
    const simulationData = new SimulationData();
    const operatorDamageTotalData: number[] = [];
    const operatorDPSTotalData: number[] = [];

    const operatorUnit = ALL_OPERATORS.find((operator) => operator.operator.id === operatorData.operatorId);
    if (!operatorUnit) throw new Error(`Operator ${operatorData.operatorId} not found`);

    operatorUnit.updateAttributeStats(operatorData.metadata, operatorData.level, operatorData.moduleData);

    const skill = operatorUnit.skills.find((skill) => skill.skillData.skillId === skillData.skillId);
    if (!skill) throw new Error(`Skill ${skillData.skillId} not found for operator ${operatorUnit.operator.name}`);

    const skillLevel = skill.skillData.levels[skillData.levelIndex];

    let damageData = 0;
    let dpsData = 0;

    const operatorAPS = formula.attacksPerSecond(operatorUnit.stats, skillLevel.modifiers.attackSpeedModifiers, skillLevel.modifiers.attackIntervalModifiers);
    const operatorFinalAttack = formula.finalAttack(operatorUnit.stats, skillLevel.modifiers.baseAttackModifiers, skillLevel.modifiers.attackMultiplierModifiers, 0);

    switch (operatorUnit.stats.attackType) {
        case AttackType.physical:
            damageData = formula.physicalDamage(operatorFinalAttack, enemyUnit, skillLevel.modifiers.flatDefModifiers, skillLevel.modifiers.scalingDefModifiers, skillLevel.modifiers.physTakenModifiers);
            dpsData = formula.damagePerSecond(operatorAPS, damageData);
            break;
        case AttackType.arts:
            damageData = formula.artsDamage(operatorFinalAttack, enemyUnit, skillLevel.modifiers.flatResModifiers, skillLevel.modifiers.scalingResModifiers, skillLevel.modifiers.artsTakenModifiers);
            dpsData = formula.damagePerSecond(operatorAPS, damageData);
            break;
        case AttackType.healing:
            damageData = operatorFinalAttack;
            dpsData = formula.damagePerSecond(operatorAPS, operatorFinalAttack);
            break;
        default:
            throw new Error(`Invalid attack type ${operatorUnit.stats.attackType}`);
    }

    if ((skillLevel.modifiers.extraPhysDamageDone ?? []).length > 0) {
        damageData += formula.physicalDamage(skillLevel.modifiers?.extraPhysDamageDone?.[0] ?? 0, enemyUnit, skillLevel.modifiers.flatDefModifiers, skillLevel.modifiers.scalingDefModifiers, skillLevel.modifiers.physTakenModifiers);
        dpsData = formula.damagePerSecond(operatorAPS, damageData);
    } else if ((skillLevel.modifiers.extraArtsDamageDone ?? []).length > 0) {
        damageData += formula.artsDamage(skillLevel.modifiers?.extraArtsDamageDone?.[0] ?? 0, enemyUnit, skillLevel.modifiers.flatResModifiers, skillLevel.modifiers.scalingResModifiers, skillLevel.modifiers.artsTakenModifiers);
        dpsData = formula.damagePerSecond(operatorAPS, damageData);
    } else if ((skillLevel.modifiers.extraTrueDamageDone ?? []).length > 0) {
        damageData += skillLevel.modifiers?.extraTrueDamageDone?.[0] ?? 0;
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
