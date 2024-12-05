import type { Modifiers, SkillData } from "../../../../../../../../types/impl/lib/impl/dps-calculator";
import { Operator } from "../../../../../../../../types/impl/lib/impl/local/impl/gamedata/impl/operators";
import Skills from "../../../../classes/skills";
import { transformSkill } from "../../../../helper/transformSkill";

export const stainless = (skill: Operator["skills"][number]) => {
    const skills: Skills[] = [];

    if (skill.skillId === "skchr_ironmn_1") {
        const skillData = transformSkill(skill);

        const newSkill: SkillData = {
            ...skillData,
            levels: skillData.levels.map((level, index) => {
                const modifiers: Modifiers = {
                    baseAttackModifiers: [0.12],
                };
                if (index === 0) {
                    modifiers.attackMultiplierModifiers = [1.1];
                } else if (index === 1) {
                    modifiers.attackMultiplierModifiers = [1.2];
                } else if (index === 2) {
                    modifiers.attackMultiplierModifiers = [1.3];
                } else if (index === 3) {
                    modifiers.attackMultiplierModifiers = [1.4];
                } else if (index === 4) {
                    modifiers.attackMultiplierModifiers = [1.5];
                } else if (index === 5) {
                    modifiers.attackMultiplierModifiers = [1.6];
                } else if (index === 6) {
                    modifiers.attackMultiplierModifiers = [1.7];
                } else if (index === 7) {
                    modifiers.attackMultiplierModifiers = [1.8];
                } else if (index === 8) {
                    modifiers.attackMultiplierModifiers = [1.9];
                } else if (index === 9) {
                    modifiers.attackMultiplierModifiers = [2];
                }

                return {
                    ...level,
                    modifiers,
                };
            }),
        };

        skills.push(new Skills(skill.static?.levels[0].name ?? "", newSkill));
    } else if (skill.skillId === "skchr_ironmn_2") {
        const skillData = transformSkill(skill);

        const newSkill: SkillData = {
            ...skillData,
            levels: skillData.levels.map((level, index) => {
                const modifiers: Modifiers = {};
                if (index === 0) {
                    modifiers.baseAttackModifiers = [0.1];
                } else if (index === 1) {
                    modifiers.baseAttackModifiers = [0.15];
                } else if (index === 2) {
                    modifiers.baseAttackModifiers = [0.2];
                } else if (index === 3) {
                    modifiers.baseAttackModifiers = [0.25];
                } else if (index === 4) {
                    modifiers.baseAttackModifiers = [0.3];
                } else if (index === 5) {
                    modifiers.baseAttackModifiers = [0.35];
                } else if (index === 6) {
                    modifiers.baseAttackModifiers = [0.4];
                } else if (index === 7) {
                    modifiers.baseAttackModifiers = [0.45];
                } else if (index === 8) {
                    modifiers.baseAttackModifiers = [0.5];
                } else if (index === 9) {
                    modifiers.baseAttackModifiers = [0.6];
                }

                return {
                    ...level,
                    modifiers,
                };
            }),
        };

        skills.push(new Skills(skill.static?.levels[0].name ?? "", newSkill));
    } else if (skill.skillId === "skchr_ironmn_3") {
        const skillData = transformSkill(skill);

        const newSkill: SkillData = {
            ...skillData,
            levels: skillData.levels.map((level, index) => {
                const modifiers: Modifiers = {};
                if (index === 0) {
                    modifiers.baseAttackModifiers = [0.1];
                    modifiers.attackSpeedModifiers = [10];
                } else if (index === 1) {
                    modifiers.baseAttackModifiers = [0.15];
                    modifiers.attackSpeedModifiers = [15];
                } else if (index === 2) {
                    modifiers.baseAttackModifiers = [0.2];
                    modifiers.attackSpeedModifiers = [20];
                } else if (index === 3) {
                    modifiers.baseAttackModifiers = [0.25];
                    modifiers.attackSpeedModifiers = [25];
                } else if (index === 4) {
                    modifiers.baseAttackModifiers = [0.3];
                    modifiers.attackSpeedModifiers = [30];
                } else if (index === 5) {
                    modifiers.baseAttackModifiers = [0.35];
                    modifiers.attackSpeedModifiers = [35];
                } else if (index === 6) {
                    modifiers.baseAttackModifiers = [0.4];
                    modifiers.attackSpeedModifiers = [40];
                } else if (index === 7) {
                    modifiers.baseAttackModifiers = [0.45];
                    modifiers.attackSpeedModifiers = [45];
                } else if (index === 8) {
                    modifiers.baseAttackModifiers = [0.5];
                    modifiers.attackSpeedModifiers = [50];
                } else if (index === 9) {
                    modifiers.baseAttackModifiers = [0.55];
                    modifiers.attackSpeedModifiers = [55];
                }

                return {
                    ...level,
                    modifiers,
                };
            }),
        };

        skills.push(new Skills(skill.static?.levels[0].name ?? "", newSkill));
    }

    return skills;
};
