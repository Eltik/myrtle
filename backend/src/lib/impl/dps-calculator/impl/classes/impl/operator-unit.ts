import { OperatorParams, OperatorTalentParameter } from "../../../../../../types/impl/lib/impl/dps-calculator";
import { BattleEquip } from "../../../../../../types/impl/lib/impl/local/impl/gamedata/impl/modules";
import { Operator, OperatorPosition, OperatorProfession } from "../../../../../../types/impl/lib/impl/local/impl/gamedata/impl/operators";
import { getDrone } from "../../../../local/impl/gamedata/impl/operators";
import { getOperatorAttributeStats } from "../../helper/getAttributeStats";
import { operatorPhaseToNumber } from "../../helper/operatorPhaseToNumber";
import { operatorRarityToNumber } from "../../helper/operatorRarityToNumber";

export default class OperatorUnit {
    /**
     * @description Constructor data.
     */
    public operatorData: Operator;
    private params: OperatorParams;

    /**
     * @description Number of targets.
     */
    private targets: number;

    /**
     * @description Operator trait details.
     */
    private atk: number;
    private atkTrust: number;
    private atkPotential: [number, number] = [0, 0];
    private atkModule = [];

    private attackInterval: number;

    private attackSpeed: number;
    private aspdTrust: number;
    private aspdPotential: [number, number] = [0, 0];
    private aspdModule = [];

    private elite: number;
    private level: number;
    private potential: number;
    private skillLevel: number;
    private trust: number;
    private operatorModule: Operator["modules"][number] | null = null;
    private operatorModuleLevel: number = -1;

    private skillIndex: number;
    private skillParameters: number[] = [];
    private skillDuration: number = -1;
    private skillCost: number = -1;

    private spBoost: number;
    private isPhysical: boolean;
    private isRanged: boolean;

    /**
     * @description Talent information
     */
    private hasSecondTalent: boolean;
    private talent1Name: string = "";
    private talent2Name: string = "";
    private talent1Parameters: number[] = [];
    private talent2Parameters: number[] = [];
    private talent1Defaults: number[] = [];
    private talent2Defaults: number[] = [];

    /**
     * @description Mech-accord caster-specific damage.
     */
    private droneAtk: number = 0;
    private droneAtkInterval: number = 0;

    /**
     * @description Whether the operator has conditional damage.
     */
    private traitDamage: boolean;
    private talentDamage: boolean;
    private talent2Damage: boolean;
    private skillDamage: boolean;
    private moduleDamage: boolean;

    /**
     * @description Buffs
     */
    private buffName: string = "";
    private buffATK: number;
    private buffATKFlat: number;
    private buffFragile: number;

    /**
     * @constructor OperatorUnit
     *
     * @param operatorData  Operator data.
     * @param params      Operator parameters.
     * @param defaultSkillIndex  Default skill index.
     * @param defaultPotential  Default potential.
     * @param defaultModIndex  Default module index.
     */
    constructor(operatorData: Operator, params: OperatorParams, defaultSkillIndex: number = 2, defaultPotential: number = 1, defaultModIndex: number = -1) {
        if (!params.allCond) params.allCond = false;
        if (!params.baseBuffs)
            params.baseBuffs = {
                atk: 1,
                atkFlat: 0,
            };
        if (!params.buffs)
            params.buffs = {
                aspd: 0,
                atk: 0,
                atkFlat: 0,
                fragile: 0,
            };
        if (!params.conditionals)
            params.conditionals = {
                moduleDamage: true,
                skillDamage: true,
                talent2Damage: true,
                talentDamage: true,
                traitDamage: true,
            };
        if (!params.def) params.def = [-1];
        if (!params.enemies) params.enemies = [];
        if (!params.fixValue) params.fixValue = 40;
        if (!params.graphType) params.graphType = 0;
        if (!params.level) params.level = -1;
        if (!params.masteryLevel) params.masteryLevel = -1;
        if (!params.maxDef) params.maxDef = 3000;
        if (!params.maxRes) params.maxRes = 120;
        if (!params.moduleIndex) params.moduleIndex = -1;
        if (!params.moduleLevel) params.moduleLevel = -1;
        if (!params.normalDPS) params.normalDPS = 0;
        if (!params.potential) params.potential = -1;
        if (!params.promotion) params.promotion = -1;
        if (!params.res) params.res = [-1];
        if (!params.shred)
            params.shred = {
                def: 1,
                defFlat: 0,
                res: 1,
                resFlat: 0,
            };
        if (!params.skillIndex) params.skillIndex = -1;
        if (!params.spBoost) params.spBoost = 0;
        if (!params.targets) params.targets = -1;
        if (!params.trust) params.trust = 100;

        this.operatorData = operatorData;
        this.params = params;

        const maxLevels = [
            [30, 30, 40, 45, 50, 50],
            [0, 0, 55, 60, 70, 80],
            [0, 0, 0, 70, 80, 90],
        ]; // E0, E1, E2 -> Rarity
        const maxPromotions = [0, 0, 1, 2, 2, 2]; // Rarity
        const maxSkillLevels = [4, 7, 10]; // Promotion level

        this.spBoost = this.params.spBoost!;

        /**
         * @description Filter out damage type.
         */
        this.isPhysical = true;
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
         * @description Filter out ranged type.
         */
        this.isRanged = true;
        if (operatorData.position === OperatorPosition.MELEE) {
            this.isRanged = false;
        }

        const operatorRarity = operatorData.rarity;
        const operatorRarityAsNumber = operatorRarityToNumber(operatorRarity);

        /**
         * @description Operator elite level.
         */
        let elite = params.promotion < 0 ? 2 : params.promotion;
        elite = Math.max(0, Math.min(maxPromotions[operatorRarityAsNumber - 1] ?? 0, elite));
        this.elite = elite;

        /**
         * @description Operator level.
         */
        const level = params.level > 0 && maxLevels[elite][operatorRarityAsNumber - 1] ? params.level : maxLevels[elite][operatorRarityAsNumber - 1];
        this.level = level;

        /**
         * @description Operator potential.
         */
        let potential = params.potential;
        if (potential < 1 || potential > 7) {
            if (defaultPotential >= 1 && defaultPotential <= 7) {
                potential = defaultPotential;
            } else {
                potential = 1;
            }
        }
        this.potential = potential;

        /**
         * @description Operator skill index.
         */
        this.skillIndex = -1;

        if (operatorRarityAsNumber > 2) {
            let skillIndex = params.skillIndex;

            if (skillIndex === -1 || operatorData.skills.length <= skillIndex) {
                if (operatorData.skills.length > defaultSkillIndex) {
                    skillIndex = defaultSkillIndex;
                } else {
                    skillIndex = operatorData.skills.length - 1;
                }
            }

            if ((elite === 1 || operatorRarityAsNumber < 6) && skillIndex === 2) {
                if (operatorData.skills.length > 1) {
                    skillIndex = 1;
                } else if (operatorData.skills.length > 0) {
                    skillIndex = 0;
                } else {
                    skillIndex = -1;
                }
            }

            if (elite === 0 && skillIndex > 0) {
                if (operatorData.skills.length > 0) {
                    skillIndex = 0;
                } else {
                    skillIndex = -1;
                }
            }

            this.skillIndex = skillIndex;
        }

        /**
         * @description Skill level
         */
        const skillLevel = params.masteryLevel > 0 && params.masteryLevel < maxSkillLevels[elite] ? params.masteryLevel : maxSkillLevels[elite];
        this.skillLevel = skillLevel;

        /**
         * @description Trust
         */
        const trust = params.trust >= 0 && params.trust < 100 ? params.trust : 100;
        this.trust = trust;

        /**
         * @description Set the conditional damage flags.
         */
        this.traitDamage = this.params.conditionals!.traitDamage ?? false;
        this.talentDamage = this.params.conditionals!.talentDamage ?? false;
        this.talent2Damage = this.params.conditionals!.talent2Damage ?? false;
        this.skillDamage = this.params.conditionals!.skillDamage ?? false;
        this.moduleDamage = this.params.conditionals!.moduleDamage ?? false;

        this.targets = this.params.targets! > 1 ? this.params.targets! : 1;

        /**
         * @description Set ATKSPD.
         */
        this.attackSpeed = 100;

        /**
         * @description Set base ATK.
         */
        const e0Phase = {
            min: operatorData.phases[0].attributesKeyFrames[0].data,
            max: operatorData.phases[0].attributesKeyFrames[1].data,
        };

        this.atk = e0Phase.min.atk + ((e0Phase.max.atk - e0Phase.min.atk) * (level - 1)) / (maxLevels[elite][operatorRarityAsNumber - 1] - 1);

        if (elite === 1) {
            const e1Phase = {
                min: operatorData.phases[1].attributesKeyFrames[0].data,
                max: operatorData.phases[1].attributesKeyFrames[1].data,
            };

            this.atk = e1Phase.min.atk + ((e1Phase.max.atk - e1Phase.min.atk) * (level - 1)) / (maxLevels[elite][operatorRarityAsNumber - 1] - 1);
        }

        if (elite === 2) {
            const e2Phase = {
                min: operatorData.phases[2].attributesKeyFrames[0].data,
                max: operatorData.phases[2].attributesKeyFrames[1].data,
            };

            this.atk = e2Phase.min.atk + ((e2Phase.max.atk - e2Phase.min.atk) * (level - 1)) / (maxLevels[elite][operatorRarityAsNumber - 1] - 1);
        }

        /**
         * @description Set ATK and ASPD trust values.
         */
        this.atkTrust = operatorData.favorKeyFrames[1].data.atk;
        this.aspdTrust = operatorData.favorKeyFrames[1].data.attackSpeed;

        /**
         * @description Module & module level
         */
        if (elite === 2 && level >= maxLevels[2][operatorRarityAsNumber - 1] - 30) {
            const availableModules = operatorData.modules;
            const operatorModule = availableModules[defaultModIndex];
            if (defaultModIndex !== -1 && !operatorModule) throw new Error(`Module not found for operator ${operatorData.name}`);
            const moduleLevel = [1, 2, 3].includes(params.moduleLevel) ? params.moduleLevel : 3;

            this.operatorModule = operatorModule;
            this.operatorModuleLevel = moduleLevel;

            /**
             * @description Update ATK and ASPD if modules add it.
             */
            if (operatorModule && operatorModule.data) {
                this.atk += operatorModule.data.phases[moduleLevel - 1].attributeBlackboard.find((data) => data.key === "atk")?.value ?? 0;
                this.attackSpeed += operatorModule.data.phases[moduleLevel - 1].attributeBlackboard.find((data) => data.key === "attack_speed")?.value ?? 0;
            }
        }

        /**
         * @description Set attack interval
         */
        const battleEquip: BattleEquip | undefined =
            this.operatorModule?.id && this.operatorModule.data
                ? {
                      [this.operatorModule?.id ?? ""]: this.operatorModule?.data,
                  }
                : undefined;

        const attributes = getOperatorAttributeStats(
            operatorData,
            {
                moduleId: this.operatorModule?.id ?? "",
                favorPoint: this.trust,
                moduleLevel: this.operatorModuleLevel ?? -1,
                phaseIndex: params.promotion,
                potentialRank: this.potential,
            },
            level,
            battleEquip,
        );
        this.attackInterval = attributes?.baseAttackTime ?? 1;

        /**
         * @description Account for potential buffs.
         */
        for (let i = 0; i < operatorData.potentialRanks.length; i++) {
            const potential = operatorData.potentialRanks[i];
            if (potential.type !== "BUFF") continue;

            /**
             * @description Set ATK potential.
             */
            if (potential.buff.attributes.attributeModifiers?.[0].attributeType === "ATK") {
                this.atkPotential = [i + 2, potential.buff.attributes.attributeModifiers?.[0].value];
            }

            /**
             * @description Set ASPD potential.
             */
            if (potential.buff.attributes.attributeModifiers?.[0].attributeType === "ATTACK_SPEED") {
                this.aspdPotential = [i + 2, potential.buff.attributes.attributeModifiers?.[0].value];
            }
        }

        /**
         * @description Set skill parameters.
         */
        const skillParams = [];
        const skillDurs = [];
        const skillSPCosts = [];
        for (const skill of operatorData.skills) {
            const currentSkillParams = [];
            const currentSkillDurations = [];
            const currentSkillSPCosts = [];

            for (const skillLevel of skill.static?.levels ?? []) {
                currentSkillDurations.push(skillLevel.duration);
                currentSkillSPCosts.push(skillLevel.spData.spCost);

                const currentInput = [];
                for (const entry of skillLevel.blackboard) {
                    currentInput.push(entry.value);
                }
                currentSkillParams.push(currentInput);
            }

            skillParams.push(currentSkillParams);
            skillDurs.push(currentSkillDurations);
            skillSPCosts.push(currentSkillSPCosts);
        }

        if (operatorRarityAsNumber > 2) {
            this.skillParameters = skillParams[this.skillIndex][this.skillLevel - 1];
            this.skillCost = skillSPCosts[this.skillIndex][this.skillLevel - 1];
            this.skillDuration = skillDurs[this.skillIndex][this.skillLevel - 1];
        }

        /**
         * @description Set talent parameters.
         */
        this.talent1Name = operatorData.talents[0]?.candidates[0].name;

        this.hasSecondTalent = operatorData.talents.length > 1;

        const talent1Parameters: OperatorTalentParameter[] = [];
        const talent2Parameters: OperatorTalentParameter[] = [];

        for (const candidate of operatorData.talents[0].candidates) {
            const params: OperatorTalentParameter = {
                requiredPromotion: operatorPhaseToNumber(candidate.unlockCondition.phase) - 1,
                requiredLevel: candidate.unlockCondition.level,
                requiredModuleId: "",
                requiredModuleLevel: -1,
                requiredPotential: candidate.requiredPotentialRank,
                talentData: candidate.blackboard.map((data) => data.value),
            };
            talent1Parameters.push(params);
        }

        if (this.hasSecondTalent) {
            this.talent2Name = operatorData.talents[1].candidates[0].name;

            for (const candidate of operatorData.talents[1].candidates) {
                const params: OperatorTalentParameter = {
                    requiredPromotion: operatorPhaseToNumber(candidate.unlockCondition.phase) - 1,
                    requiredLevel: candidate.unlockCondition.level,
                    requiredModuleId: "",
                    requiredModuleLevel: -1,
                    requiredPotential: candidate.requiredPotentialRank,
                    talentData: candidate.blackboard.map((data) => data.value),
                };
                talent2Parameters.push(params);
            }
        }

        if (this.operatorModule && this.operatorModule.data && this.operatorModuleLevel > 0) {
            const modulePhase = this.operatorModule.data.phases[this.operatorModuleLevel - 1];
            for (const part of modulePhase.parts) {
                if (part.target === "TALENT" || part.target === "TALENT_DATA_ONLY") {
                    for (const candidate of part.addOrOverrideTalentDataBundle.candidates ?? []) {
                        const requiredPromotion = 2;
                        const requiredLevel = candidate.unlockCondition.level;
                        const requiredPotential = candidate.requiredPotentialRank;
                        const requiredModuleId = this.operatorModule.id ?? "";
                        const talentData = candidate.blackboard.map((data) => data.value);

                        if (["1, 2"].includes(candidate.prefabKey ?? "")) {
                            if (candidate.name === this.talent1Name) {
                                talent1Parameters.push({
                                    requiredPromotion,
                                    requiredLevel,
                                    requiredModuleId,
                                    requiredModuleLevel: requiredLevel,
                                    requiredPotential,
                                    talentData,
                                });
                            }
                        } else if (candidate.name === this.talent2Name) {
                            talent2Parameters.push({
                                requiredPromotion,
                                requiredLevel,
                                requiredModuleId,
                                requiredModuleLevel: requiredLevel,
                                requiredPotential,
                                talentData,
                            });
                        }
                    }
                }
            }
        }

        /**
         * @description Set defaults for E0/E1.
         */
        for (const data of operatorData.talents[0].candidates.slice(-1)[0].blackboard) {
            if (["atk", "prob", "duration", "attack_speed", "attack@prob", "magic_resistance", "sp_recovery_per_sec", "base_attack_time", "magic_resist_penetrate_fixed"].includes(data.key)) {
                this.talent1Defaults.push(0);
            } else {
                this.talent1Defaults.push(1);
            }
        }

        if (this.hasSecondTalent) {
            for (const data of operatorData.talents[1].candidates.slice(-1)[0].blackboard) {
                if (["atk", "prob", "duration", "attack_speed", "attack@prob", "magic_resistance", "sp_recovery_per_sec", "base_attack_time", "magic_resist_penetrate_fixed"].includes(data.key)) {
                    this.talent2Defaults.push(0);
                } else {
                    this.talent2Defaults.push(1);
                }
            }
        }

        /**
         * @description Update talent 1 parameters
         */
        this.talent1Parameters = this.talent1Defaults;

        if (this.talent1Defaults.length > 0) {
            let currentPromo = 0;
            let currentReqLevel = 0;
            let currentReqPot = 0;
            let currentReqModuleLvl = 0;

            for (const talentData of talent1Parameters) {
                if (elite >= talentData.requiredPromotion && talentData.requiredPromotion >= currentPromo) {
                    if (level >= talentData.requiredLevel && talentData.requiredLevel >= currentReqLevel) {
                        if (!this.operatorModule) {
                            if (talentData.requiredModuleId.length === 0) {
                                if (potential > talentData.requiredPotential && potential > currentReqPot) {
                                    currentPromo = talentData.requiredPromotion;
                                    currentReqLevel = talentData.requiredLevel;
                                    currentReqPot = talentData.requiredPotential;
                                    currentReqModuleLvl = talentData.requiredModuleLevel;
                                }
                            }
                        } else {
                            let requiredModuleId;

                            if (talentData.requiredModuleId.length === 0) {
                                requiredModuleId = "";
                            } else {
                                requiredModuleId = operatorData.modules.find((module) => module.id === talentData.requiredModuleId)?.id ?? operatorData.modules[0].id ?? "";
                            }

                            if (this.operatorModule.id === requiredModuleId || talentData.requiredModuleId.length === 0) {
                                if (0 >= talentData.requiredModuleLevel && 0 >= currentReqModuleLvl) {
                                    if (potential > talentData.requiredPotential && potential > currentReqPot) {
                                        this.talent1Parameters = talentData.talentData;
                                        currentPromo = talentData.requiredPromotion;
                                        currentReqLevel = talentData.requiredLevel;
                                        currentReqPot = talentData.requiredPotential;
                                        currentReqModuleLvl = talentData.requiredModuleLevel;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        /**
         * @description Update talent 2 parameters
         */
        this.talent2Parameters = this.talent2Defaults;

        if (this.talent2Defaults.length > 0) {
            let currentPromo = 0;
            let currentReqLevel = 0;
            let currentReqPot = 0;
            let currentReqModuleLvl = 0;

            for (const talentData of talent2Parameters) {
                if (elite >= talentData.requiredPromotion && talentData.requiredPromotion >= currentPromo) {
                    if (level >= talentData.requiredLevel && talentData.requiredLevel >= currentReqLevel) {
                        if (!this.operatorModule) {
                            if (talentData.requiredModuleId.length === 0) {
                                if (potential > talentData.requiredPotential && potential > currentReqPot) {
                                    currentPromo = talentData.requiredPromotion;
                                    currentReqLevel = talentData.requiredLevel;
                                    currentReqPot = talentData.requiredPotential;
                                    currentReqModuleLvl = talentData.requiredModuleLevel;
                                }
                            }
                        } else {
                            let requiredModuleId;

                            if (talentData.requiredModuleId.length === 0) {
                                requiredModuleId = "";
                            } else {
                                requiredModuleId = operatorData.modules.find((module) => module.id === talentData.requiredModuleId)?.id ?? operatorData.modules[0].id ?? "";
                            }

                            if (this.operatorModule.id === requiredModuleId || talentData.requiredModuleId.length === 0) {
                                if (0 >= talentData.requiredModuleLevel && 0 >= currentReqModuleLvl) {
                                    if (potential > talentData.requiredPotential && potential > currentReqPot) {
                                        this.talent2Parameters = talentData.talentData;
                                        currentPromo = talentData.requiredPromotion;
                                        currentReqLevel = talentData.requiredLevel;
                                        currentReqPot = talentData.requiredPotential;
                                        currentReqModuleLvl = talentData.requiredModuleLevel;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        /**
         * @description Mech-accord caster-specific damage.
         */
        const x = operatorData.displayTokenDict;

        const droneAtk: {
            e0: number[][];
            e1: number[][];
            e2: number[][];
        } = {
            e0: [],
            e1: [],
            e2: [],
        };
        const droneAtkInterval = [];

        if (x) {
            const droneKeys = Object.keys(x);
            for (const key of droneKeys) {
                const data = getDrone(key);
                if (!data) continue;

                droneAtkInterval.push(data.phases[0].attributesKeyFrames[0].data.baseAttackTime);

                const droneAtk0 = [data.phases[0].attributesKeyFrames[0].data.atk, data.phases[0].attributesKeyFrames[1].data.atk];
                droneAtk.e0.push(droneAtk0);

                if (operatorRarityAsNumber > 2) {
                    const droneAtk1 = [data.phases[1].attributesKeyFrames[0].data.atk, data.phases[1].attributesKeyFrames[1].data.atk];
                    droneAtk.e1.push(droneAtk1);
                }
                if (operatorRarityAsNumber > 3) {
                    const droneAtk2 = [data.phases[2].attributesKeyFrames[0].data.atk, data.phases[2].attributesKeyFrames[1].data.atk];
                    droneAtk.e2.push(droneAtk2);
                }
            }
        }

        if (droneAtk.e0.length > 0) {
            let slot = this.skillIndex;
            if (droneAtk.e0.length < 2) {
                slot = 0;
            }

            this.droneAtkInterval = droneAtkInterval[slot];
            this.droneAtk = droneAtk.e0[slot][0] + ((droneAtk.e0[slot][1] - droneAtk.e0[slot][0]) * (level - 1)) / (maxLevels[elite][operatorRarityAsNumber - 1] - 1);

            if (elite === 1) {
                this.droneAtk = droneAtk.e1[slot][0] + ((droneAtk.e1[slot][1] - droneAtk.e1[slot][0]) * (level - 1)) / (maxLevels[elite][operatorRarityAsNumber - 1] - 1);
            }
            if (elite === 2) {
                this.droneAtk = droneAtk.e2[slot][0] + ((droneAtk.e2[slot][1] - droneAtk.e2[slot][0]) * (level - 1)) / (maxLevels[elite][operatorRarityAsNumber - 1] - 1);
            }
        }

        /**
         * @description Set buffs.
         */
        this.atk *= params.baseBuffs.atk! + params.buffs.atk!;

        if (params.baseBuffs?.atk && params.baseBuffs.atk! > 1) {
            this.buffName += ` bAtk+${Math.floor(100 * (params.baseBuffs.atk - 0.999999))}%`;
        } else if (params.baseBuffs.atk && params.baseBuffs.atk < 1) {
            this.buffName += ` bAtk${Math.floor(100 * (params.baseBuffs.atk - 1.000001))}%`;
        }

        if (params.baseBuffs.atkFlat && params.baseBuffs.atkFlat > 0) {
            this.buffName += ` bAtk+${Math.floor(params.baseBuffs.atkFlat)}`;
        } else if (params.baseBuffs.atkFlat && params.baseBuffs.atkFlat < 0) {
            this.buffName += ` bAtk${Math.floor(params.baseBuffs.atkFlat)}`;
        }

        this.buffATK = params.buffs.atk ?? 0;
        if (this.buffATK > 0) {
            this.buffName += ` atk+${Math.floor(100 * this.buffATK)}%`;
        } else if (this.buffATK < 0) {
            this.buffName += ` atk${Math.floor(100 * this.buffATK)}%`;
        }

        this.attackSpeed += params.buffs.aspd ?? 0;
        if (params.buffs.aspd && params.buffs.aspd > 0) {
            this.buffName += ` aspd+${Math.floor(100 * params.buffs.aspd)}%`;
        } else if (params.buffs.aspd && params.buffs.aspd < 0) {
            this.buffName += ` aspd${Math.floor(100 * params.buffs.aspd)}%`;
        }

        this.buffATKFlat = params.buffs.atkFlat ?? 0;
        if (this.buffATKFlat > 0) {
            this.buffName += ` atk+${Math.floor(this.buffATKFlat)}`;
        } else if (this.buffATKFlat < 0) {
            this.buffName += ` atk${Math.floor(this.buffATKFlat)}`;
        }

        this.buffFragile = params.buffs.fragile ?? 0;
        if (this.buffFragile > 0) {
            this.buffName += ` fragile+${Math.floor(100 * this.buffFragile)}%`;
        } else if (this.buffFragile < 0) {
            this.buffName += ` fragile${Math.floor(100 * this.buffFragile)}%`;
        }

        if (this.spBoost > 0) {
            this.buffName += ` +${this.spBoost}SP/s`;
        }

        if (params.shred.def && params.shred.def !== 1) {
            this.buffName += ` -${Math.floor(100 * (1 - params.shred.def))}%`;
        }
        if (params.shred.defFlat && params.shred.defFlat !== 0) {
            this.buffName += ` -${Math.floor(params.shred.defFlat)}`;
        }
        if (params.shred.res && params.shred.res !== 1) {
            this.buffName += ` -${Math.floor(100 * (1 - params.shred.res))}%`;
        }
        if (params.shred.resFlat && params.shred.resFlat !== 0) {
            this.buffName += ` -${Math.floor(params.shred.resFlat)}`;
        }
    }

    public normalAttack(
        enemy: {
            defense: number;
            res: number;
        },
        extraBuffs: {
            atk: number;
            flatATK: number;
            aspd: number;
        } = {
            atk: 0,
            flatATK: 0,
            aspd: 0,
        },
        hits: number = 1,
        aoe: number = 1,
    ) {
        const finalAtk = this.atk * (1 + extraBuffs.atk + this.buffATK) + extraBuffs.flatATK + this.buffATKFlat;
        let hitDmg = 0;

        if (!this.isPhysical) {
            hitDmg = Math.max(finalAtk * (1 - enemy.res / 100), finalAtk * 0.05);
        } else {
            hitDmg = Math.max(finalAtk - enemy.defense, finalAtk * 0.05);
        }

        const dps = ((hits * hitDmg) / this.attackInterval) * ((this.attackSpeed + extraBuffs.aspd) / 100) * aoe;
        return dps;
    }
}
