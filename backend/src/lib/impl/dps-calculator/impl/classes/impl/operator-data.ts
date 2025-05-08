import type { OperatorTalentParameter } from "../../../../../../types/impl/lib/impl/dps-calculator";
import { ModuleTarget } from "../../../../../../types/impl/lib/impl/local/impl/gamedata/impl/modules";
import { OperatorPosition, OperatorProfession, type Operator } from "../../../../../../types/impl/lib/impl/local/impl/gamedata/impl/operators";
import { getDrone } from "../../../../local/impl/gamedata/impl/operators";
import { operatorPhaseToNumber } from "../../helper/operatorPhaseToNumber";
import { operatorRarityToNumber } from "../../helper/operatorRarityToNumber";

export class OperatorData {
    public data: Operator;

    public rarity = 6;
    public attackInterval = 1.6;
    public availableModules: Operator["modules"] = [];

    public isPhysical = true;
    public isRanged = true;

    /**
     * @description ATK values
     */
    public atk = {
        e0: {
            min: 0,
            max: 0,
        },
        e1: {
            min: 0,
            max: 0,
        },
        e2: {
            min: 0,
            max: 0,
        },
    };

    public atkPotential = {
        requiredPotential: 0,
        value: 0,
    };

    public atkModule: {
        moduleId: string;
        value: number;
        level: number;
    }[] = [];

    public atkTrust = 0; // Max value only

    /**
     * @description ASPD values
     */
    public aspdPotential = {
        requiredPotential: 0,
        value: 0,
    };

    public aspdModule: {
        moduleId: string;
        value: number;
        level: number;
    }[] = [];

    public aspdTrust = 0;

    /**
     * @description Skill data.
     */
    public skillParameters: number[][][] = [];
    public skillDurations: number[][] = [];
    public skillCosts: number[][] = [];

    /**
     * @description Talent data.
     */
    public hasSecondTalent = true;

    public talent1Parameters: OperatorTalentParameter[] = [];
    public talent2Parameters: OperatorTalentParameter[] = [];

    public talent1Defaults: number[] = [];
    public talent2Defaults: number[] = [];

    public talent1ModuleExtra: {
        requiredModuleLevel: number;
        talentData: number[];
    }[] = [];
    public talent2ModuleExtra: {
        requiredModuleLevel: number;
        talentData: number[];
    }[] = [];

    /**
     * @description Summon-specific data.
     */
    public droneAtk: {
        e0: {
            min: number;
            max: number;
        };
        e1: {
            min: number;
            max: number;
        };
        e2: {
            min: number;
            max: number;
        };
    } = {
        e0: {
            min: 0,
            max: 0,
        },
        e1: {
            min: 0,
            max: 0,
        },
        e2: {
            min: 0,
            max: 0,
        },
    };

    public droneAtkInterval: number[] = [];

    /**
     *
     * @param operatorData
     */
    constructor(operatorData: Operator) {
        this.data = operatorData;

        /**
         * @description Set operator default values.
         */
        this.rarity = operatorRarityToNumber(operatorData.rarity);
        this.attackInterval = operatorData.phases[0]?.attributesKeyFrames[0]?.data.baseAttackTime ?? this.attackInterval;

        /**
         * @description Read ATK values.
         */
        this.atk.e0 = {
            min: operatorData.phases[0]?.attributesKeyFrames[0]?.data.atk ?? 0,
            max: operatorData.phases[0]?.attributesKeyFrames[1]?.data.atk ?? 0,
        };
        if (this.rarity > 2) {
            this.atk.e1 = {
                min: operatorData.phases[1]?.attributesKeyFrames[0]?.data.atk ?? 0,
                max: operatorData.phases[1]?.attributesKeyFrames[1]?.data.atk ?? 0,
            };
        }
        if (this.rarity > 3) {
            this.atk.e2 = {
                min: operatorData.phases[2]?.attributesKeyFrames[0]?.data.atk ?? 0,
                max: operatorData.phases[2]?.attributesKeyFrames[1]?.data.atk ?? 0,
            };
        }

        this.atkTrust = operatorData.favorKeyFrames[1]?.data.atk ?? 0;

        /**
         * @description Set ATK and ASPD potential-related stats.
         */
        for (let i = 0; i < operatorData.potentialRanks.length; i++) {
            const potential = operatorData.potentialRanks[i];
            if (potential?.type !== "BUFF") continue;

            if (potential.buff.attributes.attributeModifiers?.[0]?.attributeType === "ATK") {
                this.atkPotential = {
                    requiredPotential: i + 2,
                    value: potential.buff.attributes.attributeModifiers[0].value,
                };
            }

            if (potential.buff.attributes.attributeModifiers?.[0]?.attributeType === "ATTACK_SPEED") {
                this.aspdPotential = {
                    requiredPotential: i + 2,
                    value: potential.buff.attributes.attributeModifiers[0].value,
                };
            }
        }

        /**
         * @description Set ASPD trust value.
         */
        this.aspdTrust = operatorData.favorKeyFrames[1]?.data.attackSpeed ?? 0;

        /**
         * @description Figure out damage type.
         */
        if (operatorData.profession === OperatorProfession.SUPPORTER || operatorData.profession === OperatorProfession.CASTER || operatorData.profession === OperatorProfession.MEDIC) {
            this.isPhysical = false;
        }
        if (operatorData.subProfessionId === "craftsman") {
            this.isPhysical = true;
        }
        if (operatorData.subProfessionId === "artsfghter") {
            this.isPhysical = false;
        }

        /**
         * @description Figure out if the operator is ranged or melee.
         */
        if (operatorData.position === OperatorPosition.MELEE) {
            this.isRanged = false;
        }

        /**
         * @description Set operator skill data.
         */
        for (const skill of operatorData.skills) {
            const currentSkillParams: number[][] = [];
            const currentSkillDurations: number[] = [];
            const currentSkillCosts: number[] = [];

            for (const skillLevel of skill.static?.levels ?? []) {
                currentSkillDurations.push(skillLevel.duration);
                currentSkillCosts.push(skillLevel.spData.spCost);

                const currentInput: number[] = [];
                for (const entry of skillLevel.blackboard) {
                    currentInput.push(entry.value);
                }
                currentSkillParams.push(currentInput);
            }

            this.skillParameters.push(currentSkillParams);
            this.skillDurations.push(currentSkillDurations);
            this.skillCosts.push(currentSkillCosts);
        }

        /**
         * @description Read talents.
         */
        this.hasSecondTalent = operatorData.talents.length > 1;

        const talent1Name = operatorData.talents[0]?.candidates[0]?.name;
        const talent2Name = this.hasSecondTalent ? operatorData.talents[1]?.candidates[0]?.name : null;

        for (const candidate of operatorData.talents[0]?.candidates ?? []) {
            const params: OperatorTalentParameter = {
                requiredPromotion: operatorPhaseToNumber(candidate.unlockCondition.phase),
                requiredLevel: candidate.unlockCondition.level,
                requiredModuleId: "",
                requiredModuleLevel: -1,
                requiredPotential: candidate.requiredPotentialRank,
                talentData: candidate.blackboard.map((data) => data.value),
            };

            this.talent1Parameters.push(params);
        }

        if (this.hasSecondTalent) {
            for (const candidate of operatorData.talents[1]?.candidates ?? []) {
                const params: OperatorTalentParameter = {
                    requiredPromotion: operatorPhaseToNumber(candidate.unlockCondition.phase),
                    requiredLevel: candidate.unlockCondition.level,
                    requiredModuleId: "",
                    requiredModuleLevel: -1,
                    requiredPotential: candidate.requiredPotentialRank,
                    talentData: candidate.blackboard.map((data) => data.value),
                };

                this.talent2Parameters.push(params);
            }
        }

        /**
         * @description Set defaults for E0/E1
         */
        for (const data of operatorData.talents[0]?.candidates.at(-1)?.blackboard ?? []) {
            if (["atk", "prob", "duration", "attack_speed", "attack@prob", "magic_resistance", "sp_recovery_per_sec", "base_attack_time", "magic_resist_penetrate_fixed"].includes(data.key)) {
                this.talent1Defaults.push(0);
            } else {
                this.talent1Defaults.push(1);
            }
        }

        if (this.hasSecondTalent) {
            for (const data of operatorData.talents[1]?.candidates.at(-1)?.blackboard ?? []) {
                if (["atk", "prob", "duration", "attack_speed", "attack@prob", "magic_resistance", "sp_recovery_per_sec", "base_attack_time", "magic_resist_penetrate_fixed"].includes(data.key)) {
                    this.talent2Defaults.push(0);
                } else {
                    this.talent2Defaults.push(1);
                }
            }
        }

        /**
         * @description Figure out modules.
         */
        const hasModule = operatorData.modules.length >= 1;
        const hasSecondModule = operatorData.modules.length >= 2;
        const hasThirdModule = operatorData.modules.length >= 3;

        for (const opModule of operatorData.modules) {
            this.availableModules.push(opModule);

            const atkData: {
                moduleId: string;
                value: number;
                level: number;
            }[] = [];
            const aspdData: {
                moduleId: string;
                value: number;
                level: number;
            }[] = [];

            if (!opModule.data) continue;

            for (const modLevel of opModule.data.phases) {
                for (const data of modLevel.attributeBlackboard) {
                    if (data.key === "atk") {
                        atkData.push({
                            moduleId: opModule.id ?? "",
                            value: data.value,
                            level: modLevel.equipLevel,
                        });
                    }
                    if (data.key === "attack_speed") {
                        aspdData.push({
                            moduleId: opModule.id ?? "",
                            value: data.value,
                            level: modLevel.equipLevel,
                        });
                    }
                }
            }

            this.atkModule.push(...atkData);
            this.aspdModule.push(...aspdData);
        }

        if (hasModule) {
            const moduleKey = `uniequip_002_${operatorData.id?.split("_")[2]}`;
            const operatorModule = operatorData.modules.find((opModule) => opModule.id === moduleKey);
            if (operatorModule) {
                for (const moduleLevel of operatorModule.data.phases ?? []) {
                    const equipLevel = moduleLevel.equipLevel;

                    for (const part of moduleLevel.parts) {
                        if (part.target === ModuleTarget.TALENT || part.target === ModuleTarget.TALENT_DATA_ONLY) {
                            for (const candidate of part.addOrOverrideTalentDataBundle.candidates ?? []) {
                                if (candidate.prefabKey === "1" || candidate.prefabKey === "2") {
                                    if (candidate.name === talent1Name) {
                                        this.talent1Parameters.push({
                                            requiredPromotion: 2,
                                            requiredLevel: candidate.unlockCondition.level,
                                            requiredModuleId: operatorModule.id ?? "",
                                            requiredModuleLevel: equipLevel,
                                            requiredPotential: candidate.requiredPotentialRank,
                                            talentData: candidate.blackboard.map((data) => data.value),
                                        });
                                    } else if (candidate.name === talent2Name) {
                                        this.talent2Parameters.push({
                                            requiredPromotion: 2,
                                            requiredLevel: candidate.unlockCondition.level,
                                            requiredModuleId: operatorModule.id ?? "",
                                            requiredModuleLevel: equipLevel,
                                            requiredPotential: candidate.requiredPotentialRank,
                                            talentData: candidate.blackboard.map((data) => data.value),
                                        });
                                    }
                                } else {
                                    if (candidate.name === talent1Name) {
                                        this.talent1ModuleExtra.push({
                                            requiredModuleLevel: equipLevel,
                                            talentData: candidate.blackboard.map((data) => data.value),
                                        });
                                    } else if (candidate.name === talent2Name) {
                                        this.talent2ModuleExtra.push({
                                            requiredModuleLevel: equipLevel,
                                            talentData: candidate.blackboard.map((data) => data.value),
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        if (hasSecondModule) {
            const moduleKey = `uniequip_003_${operatorData.id?.split("_")[2]}`;
            const operatorModule = operatorData.modules.find((opModule) => opModule.id === moduleKey);
            if (operatorModule) {
                for (const moduleLevel of operatorModule.data.phases ?? []) {
                    const equipLevel = moduleLevel.equipLevel;

                    for (const part of moduleLevel.parts) {
                        if (part.target === ModuleTarget.TALENT || part.target === ModuleTarget.TALENT_DATA_ONLY) {
                            for (const candidate of part.addOrOverrideTalentDataBundle.candidates ?? []) {
                                if (candidate.prefabKey === "1" || candidate.prefabKey === "2") {
                                    if (candidate.name === talent1Name) {
                                        this.talent1Parameters.push({
                                            requiredPromotion: 2,
                                            requiredLevel: candidate.unlockCondition.level,
                                            requiredModuleId: operatorModule.id ?? "",
                                            requiredModuleLevel: equipLevel,
                                            requiredPotential: candidate.requiredPotentialRank,
                                            talentData: candidate.blackboard.map((data) => data.value),
                                        });
                                    }
                                } else {
                                    if (candidate.name === talent1Name) {
                                        this.talent1ModuleExtra.push({
                                            requiredModuleLevel: equipLevel,
                                            talentData: candidate.blackboard.map((data) => data.value),
                                        });
                                    } else if (candidate.name === talent2Name) {
                                        this.talent2ModuleExtra.push({
                                            requiredModuleLevel: equipLevel,
                                            talentData: candidate.blackboard.map((data) => data.value),
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        if (hasThirdModule) {
            const moduleKey = `uniequip_004_${operatorData.id?.split("_")[2]}`;
            const operatorModule = operatorData.modules.find((opModule) => opModule.id === moduleKey);
            if (operatorModule) {
                for (const moduleLevel of operatorModule.data.phases ?? []) {
                    const equipLevel = moduleLevel.equipLevel;

                    for (const part of moduleLevel.parts) {
                        if (part.target === ModuleTarget.TALENT || part.target === ModuleTarget.TALENT_DATA_ONLY) {
                            for (const candidate of part.addOrOverrideTalentDataBundle.candidates ?? []) {
                                if (candidate.prefabKey === "1" || candidate.prefabKey === "2") {
                                    if (candidate.name === talent1Name) {
                                        this.talent1Parameters.push({
                                            requiredPromotion: 2,
                                            requiredLevel: candidate.unlockCondition.level,
                                            requiredModuleId: operatorModule.id ?? "",
                                            requiredModuleLevel: equipLevel,
                                            requiredPotential: candidate.requiredPotentialRank,
                                            talentData: candidate.blackboard.map((data) => data.value),
                                        });
                                    }
                                } else {
                                    if (candidate.name === talent1Name) {
                                        this.talent1ModuleExtra.push({
                                            requiredModuleLevel: equipLevel,
                                            talentData: candidate.blackboard.map((data) => data.value),
                                        });
                                    } else if (candidate.name === talent2Name) {
                                        this.talent2ModuleExtra.push({
                                            requiredModuleLevel: equipLevel,
                                            talentData: candidate.blackboard.map((data) => data.value),
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        const x = operatorData.displayTokenDict;
        if (x) {
            const droneKeys = Object.keys(x);
            for (const key of droneKeys) {
                const data = getDrone(key);
                if (!data) continue;

                this.droneAtkInterval.push(data.phases[0].attributesKeyFrames[0].data.baseAttackTime);

                this.droneAtk.e0.min = data.phases[0].attributesKeyFrames[0].data.atk;
                this.droneAtk.e0.max = data.phases[0].attributesKeyFrames[1].data.atk;

                if (this.rarity > 2) {
                    this.droneAtk.e1.min = data.phases[1].attributesKeyFrames[0].data.atk;
                    this.droneAtk.e1.max = data.phases[1].attributesKeyFrames[1].data.atk;
                }
                if (this.rarity > 3) {
                    this.droneAtk.e2.min = data.phases[2].attributesKeyFrames[0].data.atk;
                    this.droneAtk.e2.max = data.phases[2].attributesKeyFrames[1].data.atk;
                }
            }
        }
    }
}
