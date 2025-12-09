// Skill types

export interface SkillBlackboard {
    key: string;
    value: number;
    valueStr: string | null;
}

export interface SkillSpData {
    spType: string;
    levelUpCost: null;
    maxChargeTime: number;
    spCost: number;
    initSp: number;
    increment: number;
}

export interface SkillLevel {
    name: string;
    rangeId: string | null;
    description: string;
    skillType: string;
    durationType: string | null;
    spData: SkillSpData;
    prefabId: string;
    duration: number;
    blackboard: SkillBlackboard[];
}

export interface RawSkill {
    skillId: string;
    iconId: string | null;
    hidden: boolean;
    levels: SkillLevel[];
}

export interface Skill {
    id: string | null;
    skillId: string;
    iconId: string | null;
    image: string | null;
    hidden: boolean;
    levels: SkillLevel[];
}

export interface SkillTableFile {
    skills: Record<string, RawSkill>;
}
