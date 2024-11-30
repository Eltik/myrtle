export type Skill = {
    id?: string;
    skillId: string;
    iconId: string | null;
    image?: string;
    hidden: boolean;
    levels: {
        name: string;
        rangeId: string | null;
        description: string;
        skillType: string;
        spData: {
            spType: string;
            levelUpCost: null;
            maxChargeTime: number;
            spCost: number;
            initSp: number;
            increment: number;
        };
        prefabId: string;
        duration: number;
        blackboard: { key: string; value: number; valueStr: string | null }[];
    }[];
};
