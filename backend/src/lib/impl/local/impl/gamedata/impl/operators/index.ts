import { getHandbook, getSkill, getSkins, getVoices, modules } from "../..";
import type { Drone, Operator } from "../../../../../../../types/impl/lib/impl/local/impl/gamedata/impl/operators";
import { STATIC_DATA } from "../../../handler";
import { parseOperatorProfile } from "../../../helper";

export const getAll = (extraData: boolean = true): Operator[] => {
    const data = STATIC_DATA?.CHARACTER_TABLE as Record<string, Operator>;

    if (!extraData) return Object.entries(data).map(([id, operator]) => ({ id, ...operator }));

    const operatorData = Object.entries(data).map(([id, operator]) => {
        // Handle skills
        operator.skills.map((skill) => {
            const data = getSkill(skill.skillId);
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
        const operatorModule = modules.getByCharId(id);
        if (operatorModule) {
            operatorModule.map((module) => {
                const data = modules.getModuleDetails(module.uniEquipId);
                if (data) {
                    Object.assign(module, {
                        id: module.uniEquipId,
                        data: {
                            ...data,
                        },
                    });
                }
            });

            Object.assign(operator, {
                modules: operatorModule,
            });
        }

        // Handle handbook items
        const handbookData = getHandbook(id);
        if (handbookData) {
            try {
                const operatorProfile = parseOperatorProfile(handbookData.storyTextAudio);
                Object.assign(operator, {
                    handbook: handbookData,
                    profile: operatorProfile,
                });
            } catch (e) {
                console.error(e);
                console.error(`Failed to parse operator profile for ${id}`);
                Object.assign(operator, {
                    handbook: handbookData,
                    profile: null,
                });
            }
        }

        // Handle VA's
        const voiceData = getVoices(id);
        if (voiceData) {
            Object.assign(operator, {
                voiceActors: voiceData.map((voice) => ({
                    id: voice.id,
                    name: voice.data?.[0]?.cvName ?? null,
                    language: voice.data?.[0]?.language ?? null,
                })),
            });
        }

        // Handle artists
        const skins = getSkins(id);
        if (skins) {
            Object.assign(operator, {
                artists: skins.map((skin) => skin.displaySkin.drawerList).flat(),
            });
        }

        return {
            id,
            ...operator,
        };
    });

    // Wait for all operators to be processed
    const operators = operatorData.map((operator) => operator);
    return operators;
};

export const getDrone = (id: string): Drone | null => {
    const data = getAll(false);
    const drone = (data.find((drone) => drone.id === id) as unknown as Drone | null) ?? null;
    return drone;
};

export default (id: string): Operator | null => {
    const operators = getAll(false);
    const operator = operators.find((operator) => operator.id === id) ?? null;

    if (!operator?.id?.startsWith("char")) return null;

    if (operator) {
        // Handle skills
        operator.skills.map((skill) => {
            const data = getSkill(skill.skillId);
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
        const operatorModule = modules.getByCharId(id);
        if (operatorModule) {
            operatorModule.map((module) => {
                const data = modules.getModuleDetails(module.uniEquipId);
                if (data) {
                    Object.assign(module, {
                        id: module.uniEquipId,
                        data: {
                            ...data,
                        },
                    });
                }
            });

            Object.assign(operator, {
                modules: operatorModule,
            });
        }

        // Handle handbook items
        const handbookData = getHandbook(id);
        if (handbookData) {
            try {
                const operatorProfile = parseOperatorProfile(handbookData.storyTextAudio);
                Object.assign(operator, {
                    handbook: handbookData,
                    profile: operatorProfile,
                });
            } catch (e) {
                console.error(e);
                console.error(`Failed to parse operator profile for ${id}`);
                Object.assign(operator, {
                    handbook: handbookData,
                    profile: null,
                });
            }
        }

        // Handle artists
        const skins = getSkins(id);
        if (skins) {
            const artists: string[] = [];
            for (const skin of skins) {
                if (!skin.displaySkin.drawerList) continue;

                for (const drawer of skin.displaySkin.drawerList) {
                    if (artists.includes(drawer)) continue;
                    artists.push(drawer);
                }
            }
            Object.assign(operator, {
                artists,
            });
        }

        return {
            id,
            ...operator,
        };
    }

    return operator;
};
