import { OperatorParams } from "../../../../../../types/impl/lib/impl/dps-calculator";
import { Operator } from "../../../../../../types/impl/lib/impl/local/impl/gamedata/impl/operators";
import { OperatorData } from "./operator-data";

export class OperatorUnit {
    /**
     * @description Constructor data.
     */
    public operatorData: OperatorData;
    public params: OperatorParams;

    public defaultSkillIndex = -1;
    public defaultPotential = 1;
    public defaultModuleIndex = -1;

    /**
     * @description Number of targets.
     */
    public targets: number;

    /**
     * @description Operator trait details.
     */
    public rarity: number;

    public atk: number;

    public attackInterval: number;

    public attackSpeed: number;

    public elite: number;
    private level: number;
    public potential: number;
    public skillLevel: number;
    public trust: number;
    public operatorModule: Operator["modules"][number] | null = null;
    public operatorModuleLevel: number = -1;

    public skillIndex: number;
    public skillParameters: number[] = [];
    public skillDuration: number = -1;
    public skillCost: number = -1;

    public spBoost: number;
    public isPhysical: boolean;
    public isRanged: boolean;

    /**
     * @description Talent information
     */
    public talent1Parameters: number[] = [];
    public talent2Parameters: number[] = [];

    /**
     * @description Mech-accord caster-specific damage.
     */
    public droneAtk: number = 0;
    public droneAtkInterval: number = 0;

    /**
     * @description Whether the operator has conditional damage.
     */
    public traitDamage: boolean;
    public traitDamageName: string | undefined;
    public traitDamageNames: string[] = [];

    public talentDamage: boolean;
    public talentDamageName: string | undefined;
    public talentDamageNames: string[] = [];

    public talent2Damage: boolean;
    public talent2DamageName: string | undefined;
    public talent2DamageNames: string[] = [];

    public skillDamage: boolean;
    public skillDamageName: string | undefined;
    public skillDamageNames: string[] = [];

    public moduleDamage: boolean;
    public moduleDamageName: string | undefined;
    public moduleDamageNames: string[] = [];

    /**
     * @description Buffs
     */
    private buffName: string = "";
    public buffATK: number;
    public buffATKFlat: number;
    public buffFragile: number;

    constructor(operatorData: OperatorData, params: OperatorParams, defaultSkillIndex: number = 2, defaultPotential: number = 1, defaultModIndex: number = -1) {
        this.defaultSkillIndex = defaultSkillIndex;
        this.defaultPotential = defaultPotential;
        this.defaultModuleIndex = defaultModIndex;

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
        if (isNaN(params.skillIndex ?? -1) || params.skillIndex === undefined) params.skillIndex = -1;
        if (!params.spBoost) params.spBoost = 0;
        if (!params.targets) params.targets = -1;
        if (!params.trust) params.trust = 100;

        /**
         * @description Set default operator data and params.
         */
        this.operatorData = operatorData;
        this.params = params;

        /**
         * @description Set operator rarity.
         */
        this.rarity = operatorData.rarity;

        /**
         * @description Constants.
         */
        const maxLevels = [
            [30, 30, 40, 45, 50, 50],
            [0, 0, 55, 60, 70, 80],
            [0, 0, 0, 70, 80, 90],
        ]; // E0, E1, E2 -> Rarity
        const maxPromotions = [0, 0, 1, 2, 2, 2]; // Rarity
        const maxSkillLevels = [4, 7, 10]; // Promotion level

        /**
         * @description Set base attack interval.
         */
        this.attackInterval = operatorData.attackInterval;

        /**
         * @description Set conditionals.
         */
        this.traitDamage = params.conditionals.traitDamage ?? true;
        this.talentDamage = params.conditionals.talentDamage ?? true;
        this.talent2Damage = params.conditionals.talent2Damage ?? true;
        this.skillDamage = params.conditionals.skillDamage ?? true;
        this.moduleDamage = params.conditionals.moduleDamage ?? true;

        /**
         * @description Set targets.
         */
        this.targets = Math.max(1, params.targets);

        /**
         * @description Set SP boost.
         */
        this.spBoost = params.spBoost;

        /**
         * @description Filter out ranged type.
         */
        this.isRanged = operatorData.isRanged;

        /**
         * @description Filter out damage type.
         */
        this.isPhysical = operatorData.isRanged;

        /**
         * @description Set operator elite level.
         */
        let elite = params.promotion < 0 ? 2 : params.promotion;
        elite = Math.max(0, Math.min(maxPromotions[this.rarity - 1], elite));

        this.elite = elite;

        /**
         * @description Set operator level.
         */
        const level = params.level > 0 && params.level < maxLevels[this.elite][this.rarity - 1] ? params.level : maxLevels[this.elite][this.rarity - 1];
        this.level = level;

        /**
         * @description Set operator potential.
         */
        let potential = params.potential;
        if (!(potential >= 1 && potential <= 6)) {
            if (defaultPotential >= 1 && defaultPotential <= 6) {
                potential = defaultPotential;
            } else {
                potential = 1;
            }
        }
        this.potential = potential;

        /**
         * @description Set operator skillIndex.
         */
        this.skillIndex = -1;
        if (this.rarity > 2) {
            let skillIndex = params.skillIndex;

            if (!(operatorData.data.skills.length > skillIndex || skillIndex === -1)) {
                if (operatorData.data.skills.length > defaultSkillIndex) {
                    skillIndex = defaultSkillIndex;
                } else {
                    skillIndex = operatorData.data.skills.length - 1;
                }
            }

            // Amiya is a special child
            if (operatorData.data.id !== "char_002_amiya" && (this.elite === 1 || (this.rarity < 6 && skillIndex === 2))) {
                if (operatorData.data.skills.length >= 2) {
                    skillIndex = 1;
                } else if (operatorData.data.skills.length >= 1) {
                    skillIndex = 0;
                } else {
                    skillIndex = -1;
                }
            }

            if (this.elite === 0 && skillIndex >= 0) {
                if (operatorData.data.skills.length >= 1) {
                    skillIndex = 0;
                } else {
                    skillIndex = -1;
                }
            }

            this.skillIndex = skillIndex;
        }

        /**
         * @description Set operator skill level.
         */
        const skillLevel = params.masteryLevel > 0 && params.masteryLevel + 6 < maxSkillLevels[this.elite] ? (params.masteryLevel === 3 ? 9 : params.masteryLevel === 2 ? 8 : params.masteryLevel === 1 ? 7 : 9) : maxSkillLevels[elite];
        this.skillLevel = skillLevel;

        /**
         * @description Set operator trust.
         */
        const trust = params.trust >= 0 && params.trust < 100 ? params.trust : 100;
        this.trust = trust;

        /**
         * @description Set module level.
         */
        let moduleLevel = 0;
        if (this.elite === 2 && this.level >= maxLevels[2][this.rarity - 1] - 30) {
            let availableModules = operatorData.availableModules;
            let operatorModule: Operator["modules"][number] | null = availableModules[defaultModIndex];
            if (!operatorModule && defaultModIndex !== -1) throw new Error(`Could not find operator module for ${operatorData.data.name}.`);

            this.operatorModule = operatorModule;

            if (operatorData.atkModule.length === 0) {
                availableModules = [];
                moduleLevel = 0;
            }

            if (availableModules.length > 0) {
                if (params.moduleIndex === -1) {
                    availableModules = [];
                    operatorModule = null;
                } else {
                    if (availableModules.length > params.moduleIndex) {
                        operatorModule = availableModules[params.moduleIndex];
                    }
                    moduleLevel = params.moduleLevel <= 3 && params.moduleLevel > 1 ? params.moduleLevel : 3;
                    if (trust < 50) {
                        moduleLevel = 1;
                    }
                    if (trust < 100) {
                        moduleLevel = Math.min(2, moduleLevel);
                    }
                }
            }

            this.operatorModule = operatorModule;
            this.operatorModuleLevel = moduleLevel;
        }

        /**
         * @description Read parameters from JSON.
         */

        /**
         * @description Set default attack speed.
         */
        this.attackSpeed = 100;

        /**
         * @description Set ATK.
         */
        this.atk = operatorData.atk.e0.min + ((operatorData.atk.e0.max - operatorData.atk.e0.min) * (level - 1)) / (maxLevels[this.elite][this.rarity - 1] - 1);
        if (this.elite === 1) {
            this.atk = operatorData.atk.e1.min + ((operatorData.atk.e1.max - operatorData.atk.e1.min) * (level - 1)) / (maxLevels[this.elite][this.rarity - 1] - 1);
        }
        if (this.elite === 2) {
            this.atk = operatorData.atk.e2.min + ((operatorData.atk.e2.max - operatorData.atk.e2.min) * (level - 1)) / (maxLevels[this.elite][this.rarity - 1] - 1);
        }

        /**
         * @description Set potential stats.
         */
        if (this.potential >= operatorData.atkPotential.requiredPotential) {
            this.atk += operatorData.atkPotential.value;
        }
        this.atk += (operatorData.atkTrust * trust) / 100;

        if (this.potential >= operatorData.aspdPotential.requiredPotential) {
            this.attackSpeed += operatorData.aspdPotential.value;
        }
        this.attackSpeed += (operatorData.aspdTrust * trust) / 100;

        if (this.elite === 2 && this.level >= maxLevels[2][this.rarity - 1] - 30) {
            let highestATKValue = 0;
            let highestASPDValue = 0;

            for (const opModule of operatorData.atkModule) {
                if (opModule.moduleId === this.operatorModule?.id) {
                    highestATKValue = Math.max(highestATKValue, opModule.value);
                }
            }
            for (const opModule of operatorData.aspdModule) {
                if (opModule.moduleId === this.operatorModule?.id) {
                    highestASPDValue = Math.max(highestASPDValue, opModule.value);
                }
            }

            this.atk += highestATKValue;
            this.attackSpeed += highestASPDValue;
        }

        /**
         * @description Set skill parameters
         */
        if (this.rarity > 2) {
            this.skillParameters = operatorData.skillParameters[this.skillIndex]?.[this.skillLevel - 1] ?? [];
            this.skillCost = operatorData.skillCosts[this.skillIndex]?.[this.skillLevel - 1] ?? -1;
            this.skillDuration = operatorData.skillDurations[this.skillIndex]?.[this.skillLevel - 1] ?? -1;
        }

        /**
         * @description Set talent parameters
         */
        this.talent1Parameters = operatorData.talent1Defaults;
        if (operatorData.talent1Parameters.length > 0) {
            let currentPromo = 0;
            let currentReqLevel = 0;
            let currentReqPotential = 0;
            let currentReqModuleLvl = 0;

            for (const talentData of operatorData.talent1Parameters) {
                if (this.elite >= talentData.requiredPromotion && talentData.requiredPromotion >= currentPromo) {
                    if (this.level >= talentData.requiredLevel && talentData.requiredLevel >= currentReqLevel) {
                        if (!this.operatorModule) {
                            if (!talentData.requiredModuleId || talentData.requiredModuleId.length === 0) {
                                if (this.potential > talentData.requiredPotential && this.potential > currentReqPotential) {
                                    this.talent1Parameters = talentData.talentData;

                                    currentPromo = talentData.requiredPromotion;
                                    currentReqLevel = talentData.requiredLevel;
                                    currentReqPotential = talentData.requiredPotential;
                                    currentReqModuleLvl = talentData.requiredModuleLevel;
                                }
                            }
                        } else {
                            let requiredModuleId = "";
                            if (talentData.requiredModuleId.length === 0) {
                                requiredModuleId = "";
                            } else {
                                requiredModuleId = talentData.requiredModuleId.length > 0 ? talentData.requiredModuleId : "";
                            }

                            if (this.operatorModule.id === requiredModuleId || talentData.requiredModuleId.length === 0) {
                                if (this.operatorModuleLevel >= talentData.requiredModuleLevel && this.operatorModuleLevel >= currentReqModuleLvl) {
                                    if (this.potential > talentData.requiredPotential && this.potential > currentReqPotential) {
                                        this.talent1Parameters = talentData.talentData;

                                        currentPromo = talentData.requiredPromotion;
                                        currentReqLevel = talentData.requiredLevel;
                                        currentReqPotential = talentData.requiredPotential;
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
         * @description Set talent 2 parameters.
         */
        this.talent2Parameters = operatorData.talent2Defaults;
        if (operatorData.talent2Parameters.length > 0) {
            let currentPromo = 0;
            let currentReqLevel = 0;
            let currentReqPotential = 0;
            let currentReqModuleLvl = 0;

            for (const talentData of operatorData.talent2Parameters) {
                if (this.elite >= talentData.requiredPromotion && talentData.requiredPromotion >= currentPromo) {
                    if (this.level >= talentData.requiredLevel && talentData.requiredLevel >= currentReqLevel) {
                        if (!this.operatorModule) {
                            if (!talentData.requiredModuleId || talentData.requiredModuleId.length === 0) {
                                if (this.potential > talentData.requiredPotential && this.potential > currentReqPotential) {
                                    this.talent2Parameters = talentData.talentData;

                                    currentPromo = talentData.requiredPromotion;
                                    currentReqLevel = talentData.requiredLevel;
                                    currentReqPotential = talentData.requiredPotential;
                                    currentReqModuleLvl = talentData.requiredModuleLevel;
                                }
                            }
                        } else {
                            let requiredModuleId = "";
                            if (talentData.requiredModuleId.length === 0) {
                                requiredModuleId = "";
                            } else {
                                requiredModuleId = talentData.requiredModuleId.length > 0 ? talentData.requiredModuleId : "";
                            }

                            if (this.operatorModule.id === requiredModuleId || talentData.requiredModuleId.length === 0) {
                                if (this.operatorModuleLevel >= talentData.requiredModuleLevel && this.operatorModuleLevel >= currentReqModuleLvl) {
                                    if (this.potential > talentData.requiredPotential && this.potential > currentReqPotential) {
                                        this.talent2Parameters = talentData.talentData;

                                        currentPromo = talentData.requiredPromotion;
                                        currentReqLevel = talentData.requiredLevel;
                                        currentReqPotential = talentData.requiredPotential;
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
         * @description Set summon parameters.
         */
        this.droneAtk = 0;
        this.droneAtkInterval = 0;
        if (operatorData.droneAtk.e0.max !== 0 && operatorData.droneAtk.e0.min !== 0) {
            const slot = this.skillIndex;

            this.droneAtkInterval = operatorData.droneAtkInterval[slot];
            this.droneAtk = operatorData.droneAtk.e0.min + ((operatorData.droneAtk.e0.max - operatorData.droneAtk.e0.min) * (level - 1)) / (maxLevels[this.elite][this.rarity - 1] - 1);

            if (this.elite === 1) {
                this.droneAtk = operatorData.droneAtk.e1.min + ((operatorData.droneAtk.e1.max - operatorData.droneAtk.e1.min) * (level - 1)) / (maxLevels[this.elite][this.rarity - 1] - 1);
            }
            if (this.elite === 2) {
                this.droneAtk = operatorData.droneAtk.e2.min + ((operatorData.droneAtk.e2.max - operatorData.droneAtk.e2.min) * (level - 1)) / (maxLevels[this.elite][this.rarity - 1] - 1);
            }
        }

        /**
         * @description Buffs
         */
        this.atk = this.atk * (params.baseBuffs?.atk ?? 0) + (params.baseBuffs.atkFlat ?? 0);
        if ((params.baseBuffs?.atk ?? 0) > 1) {
            this.buffName += ` bAtk+${(100 * (params.baseBuffs?.atk ?? 0)).toFixed(0)}%`;
        } else if ((params.baseBuffs?.atk ?? 0) < 1) {
            this.buffName += ` bAtk${(100 * (params.baseBuffs?.atk ?? 0)).toFixed(0)}%`;
        }

        this.buffATK = params.buffs?.atk ?? 0;
        if ((params.buffs?.atk ?? 0) > 1) {
            this.buffName += ` atk+${(100 * (params.buffs?.atk ?? 0)).toFixed(0)}%`;
        } else if ((params.buffs?.atk ?? 0) < 1) {
            this.buffName += ` atk${(100 * (params.buffs?.atk ?? 0)).toFixed(0)}%`;
        }

        this.attackSpeed += params.buffs?.aspd ?? 0;
        if ((params.buffs?.aspd ?? 0) > 1) {
            this.buffName += ` aspd+${(100 * (params.buffs?.aspd ?? 0)).toFixed(0)}%`;
        } else if ((params.buffs?.aspd ?? 0) < 1) {
            this.buffName += ` aspd${(100 * (params.buffs?.aspd ?? 0)).toFixed(0)}%`;
        }

        this.buffATKFlat = params.buffs?.atkFlat ?? 0;
        if ((params.buffs?.atkFlat ?? 0) > 0) {
            this.buffName += ` atk+${(params.buffs?.atkFlat ?? 0).toFixed(0)}`;
        } else if ((params.buffs?.atkFlat ?? 0) < 0) {
            this.buffName += ` atk${(params.buffs?.atkFlat ?? 0).toFixed(0)}`;
        }

        this.buffFragile = params.buffs?.fragile ?? 0;
        if ((params.buffs?.fragile ?? 0) > 1) {
            this.buffName += ` dmg+${(100 * (params.buffs?.fragile ?? 0)).toFixed(0)}%`;
        } else if ((params.buffs?.fragile ?? 0) < 1) {
            this.buffName += ` dmg${(100 * (params.buffs?.fragile ?? 0)).toFixed(0)}%`;
        }

        if (this.spBoost > 0) this.buffName += ` +${this.spBoost}SP/s`;

        if (params.shred.def !== 1) {
            this.buffName += ` -shredDef${(100 * (1 - (params.shred?.def ?? 0))).toFixed(0)}%def`;
        }
        if (params.shred.defFlat !== 0) {
            this.buffName += ` -shredDef${(params.shred?.defFlat ?? 0).toFixed(0)}def`;
        }
        if (params.shred.res !== 1) {
            this.buffName += ` -shredRes${(100 * (1 - (params.shred?.res ?? 0))).toFixed(0)}%res`;
        }
        if (params.shred.resFlat !== 0) {
            this.buffName += ` -shredRes${(params.shred?.resFlat ?? 0).toFixed(0)}res`;
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

    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    public skillDPS(enemy: { defense: number; res: number }): number {
        throw new Error("Not implemented");
    }

    public totalDMG(enemy: { defense: number; res: number }) {
        if (this.skillDuration < 1 || this.skillIndex === -1) {
            return this.skillDPS(enemy);
        } else {
            return this.skillDPS(enemy) * this.skillDuration;
        }
    }

    public averageDPS(enemy: { defense: number; res: number }) {
        if (this.skillDuration < 1 || this.skillIndex === -1) {
            return this.skillDPS(enemy);
        } else {
            const tmp = this.skillIndex;
            const skillDPS = this.skillDPS(enemy);

            this.skillIndex = -1;

            const offSkillDPS = this.skillDPS(enemy);

            this.skillIndex = tmp;

            const cycleDMG = skillDPS * this.skillDuration + (offSkillDPS * this.skillCost) / (1 + this.spBoost);
            const dps = cycleDMG / (this.skillDuration + this.skillCost / (1 + this.spBoost));
            return dps;
        }
    }
}
