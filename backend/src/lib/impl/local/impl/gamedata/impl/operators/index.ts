import { getSkill, modules } from "../..";
import type { Operator } from "../../../../../../../types/impl/lib/impl/local/impl/gamedata/impl/operators";
import { ExcelTables } from "../../../../../../../types/impl/lib/impl/local/impl/handler";
import { get as getOperators } from "../../../handler/impl/get";

export const getAll = async (promisify: boolean = true): Promise<Operator[]> => {
    const data = (await getOperators(ExcelTables.CHARACTER_TABLE)) as Record<string, Operator>;

    if (!promisify) return Object.entries(data).map(([id, operator]) => ({ id, ...operator }));

    const operatorPromises = Object.entries(data).map(async ([id, operator]) => {
        // Handle skills
        const skillPromises = operator.skills.map(async (skill) => {
            const data = await getSkill(skill.skillId);
            if (data) {
                Object.assign(skill, {
                    static: {
                        levels: data.levels.map((level) => ({
                            name: level.name,
                            rangeId: level.rangeId,
                            description: level.description,
                            skillType: level.skillType,
                            duration: level.duration,
                            spData: {
                                spType: level.spData.spType,
                                levelUpCost: level.spData.levelUpCost,
                                maxChargeTime: level.spData.maxChargeTime,
                                spCost: level.spData.spCost,
                                initSp: level.spData.initSp,
                                increment: level.spData.increment,
                            },
                            prefabId: level.prefabId,
                            blackboard: level.blackboard.map((blackboard) => ({
                                key: blackboard.key,
                                value: blackboard.value,
                                valueStr: blackboard.valueStr,
                            })),
                        })),
                        skillId: data.skillId,
                        iconId: data.iconId,
                        hidden: data.hidden,
                        image: data.image,
                    },
                });
            }
        });

        // Handle modules
        const modulePromises = new Promise<void>(async (resolve) => {
            const operatorModule = await modules.getByCharId(id);
            if (operatorModule) {
                const moduleDetailsPromises = operatorModule.map(async (module) => {
                    const data = await modules.getModuleDetails(module.uniEquipId);
                    if (data) {
                        Object.assign(module, {
                            id: module.uniEquipId,
                            data: {
                                ...data,
                            },
                        });
                    }
                });

                await Promise.all(moduleDetailsPromises);
                Object.assign(operator, {
                    modules: operatorModule,
                });
            }
            resolve();
        });

        // Wait for all skill and module promises
        await Promise.all([...skillPromises, modulePromises]);

        return {
            id,
            ...operator,
        };
    });

    // Wait for all operators to be processed
    const operators = await Promise.all(operatorPromises);
    return operators;
};

export default async (id: string): Promise<Operator | null> => {
    const operators = await getAll(false);
    const operator = operators.find((operator) => operator.id === id) ?? null;

    if (!operator?.id?.startsWith("char")) return null;

    if (operator) {
        // Handle skills
        const skillPromises = operator.skills.map(async (skill) => {
            const data = await getSkill(skill.skillId);
            if (data) {
                Object.assign(skill, {
                    static: {
                        levels: data.levels.map((level) => ({
                            name: level.name,
                            rangeId: level.rangeId,
                            description: level.description,
                            skillType: level.skillType,
                            duration: level.duration,
                            spData: {
                                spType: level.spData.spType,
                                levelUpCost: level.spData.levelUpCost,
                                maxChargeTime: level.spData.maxChargeTime,
                                spCost: level.spData.spCost,
                                initSp: level.spData.initSp,
                                increment: level.spData.increment,
                            },
                            prefabId: level.prefabId,
                            blackboard: level.blackboard.map((blackboard) => ({
                                key: blackboard.key,
                                value: blackboard.value,
                                valueStr: blackboard.valueStr,
                            })),
                        })),
                        skillId: data.skillId,
                        iconId: data.iconId,
                        hidden: data.hidden,
                        image: data.image,
                    },
                });
            }
        });

        // Handle modules
        const modulePromises = new Promise<void>(async (resolve) => {
            const operatorModule = await modules.getByCharId(id);
            if (operatorModule) {
                const moduleDetailsPromises = operatorModule.map(async (module) => {
                    const data = await modules.getModuleDetails(module.uniEquipId);
                    if (data) {
                        Object.assign(module, {
                            id: module.uniEquipId,
                            data: {
                                ...data,
                            },
                        });
                    }
                });

                await Promise.all(moduleDetailsPromises);
                Object.assign(operator, {
                    modules: operatorModule,
                });
            }
            resolve();
        });

        // Wait for all skill and module promises
        await Promise.all([...skillPromises, modulePromises]);

        return {
            id,
            ...operator,
        };
    }

    return operator;
};
