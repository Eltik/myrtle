import { ALL_OPERATORS } from "../..";
import emitter, { Events } from "../../../../../../../events";
import { AttackType } from "../../../../../../../types/impl/lib/impl/dps-calculator";
import operators from "../../../../../local/impl/gamedata/impl/operators";
import OperatorUnit from "../../../classes/operator-unit";
import Skills from "../../../classes/skills";
import Stats from "../../../classes/stats";
import { getOperatorAttributeStats } from "../../../helper/getAttributeStats";
import { stainless } from "./impl/stainless";

async function fetchOperatorData(operatorId: string) {
    const operatorData = await operators(operatorId);
    if (!operatorData) throw new Error(`Operator data not found for ${operatorId}`);

    const { phases = [], modules = [], name, profession, subProfessionId } = operatorData;
    const lastPhase = phases[phases.length - 1];
    const lastModule = modules[modules.length - 1];

    const attributeStats = getOperatorAttributeStats(
        operatorData,
        {
            phaseIndex: phases.length - 1,
            favorPoint: 200,
            moduleId: lastModule?.id ?? "",
            moduleLevel: lastModule?.data.phases[lastModule.data.phases.length - 1]?.equipLevel ?? 1,
            potentialRank: 0,
        },
        lastPhase?.maxLevel ?? 90,
        modules.reduce((acc, module) => {
            Object.assign(acc, {
                [module.id ?? ""]: module.data,
            });
            return acc;
        }, {}),
    );

    const attackType = operatorId === "char_4072_ironmn" ? AttackType.physical : AttackType.physical;

    const stats = new Stats(
        name ?? "",
        attributeStats?.def ?? 0,
        attributeStats?.magicResistance ?? 0,
        profession ?? null,
        subProfessionId ?? null,
        attributeStats?.maxHp ?? 0,
        attributeStats?.atk ?? 0,
        attributeStats?.cost ?? 0,
        attributeStats?.respawnTime ?? 0,
        attributeStats?.blockCnt ?? 1,
        attributeStats?.attackSpeed ?? 0,
        attackType,
    );

    const skills: Skills[] = [];

    for (const skill of operatorData.skills) {
        switch (operatorData.id) {
            case "char_4072_ironmn":
                const stainlessSkills = stainless(skill);
                skills.push(...stainlessSkills);
        }
    }

    return new OperatorUnit(operatorData, stats, skills);
}

export default async function fetchArtificers() {
    const operatorIds = ["char_4072_ironmn", "char_484_robrta", "char_433_windft"];
    const operatorsData = await Promise.all(operatorIds.map(fetchOperatorData));

    await emitter.emit(Events.DPS_CALCULATOR_CLASS_FETCHED, {
        name: "Artificers",
    });

    ALL_OPERATORS.push(...operatorsData);
}
