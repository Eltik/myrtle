import { ItemType } from "../materials";

export type Modules = {
    equipDict: Record<string, Module>;
    missionList: Record<string, Mission>;
    subProfDict: Record<
        string,
        {
            subProfessionId: string;
            subProfessionName: string;
            subProfessionCatagory: number;
        }
    >;
    charEquip: Record<string, string[]>;
    equipTrackDict: Record<
        string,
        {
            timestamp: number;
            trackList: {
                charId: string;
                equipId: string;
            }[];
        }
    >;
};

export type Module = {
    id?: string;
    uniEquipId: string;
    uniEquipName: string;
    uniEquipIcon: string;
    image?: string;
    uniEquipDesc: string;
    typeIcon: string;
    typeName1: string;
    typeName2: string | null; // If it's Y, X, etc.
    equipShiningColor: string; // ex. grey
    showEvolvePhase: string;
    unlockEvolvePhase: string;
    charId: string;
    tmplId: string | null;
    showLevel: number;
    unlockLevel: number;
    unlockFavorPoint: number;
    missionList: string[];
    itemCost: Record<
        string,
        {
            id: string;
            count: number;
            type: ItemType;
        }
    > | null;
    type: ModuleType;
    uniEquipGetTime: number;
    charEquipOrder: number;
};

export enum ModuleType {
    INITIAL = "INITIAL",
    ADVANCED = "ADVANCED",
}

export type Mission = {
    template: string;
    desc: string;
    paramList: string[];
    uniEquipMissionId: string;
    uniEquipMissionSort: number;
    uniEquipId: string;
    jumpStageId: string | null;
};

export type BattleEquip = Record<string, ModuleData>;
export type ModuleData = {
    phases: {
        equipLevel: number;
        parts: {
            resKey: string;
            target: ModuleTarget;
            isToken: boolean;
            addOrOverrideTalentDataBundle: {
                candidates: AddModuleCandidates[] | null;
            };
            overrideTraitDataBundle: {
                candidates: ModuleCandidates[] | null;
            };
        }[];
        attributeBlackboard: {
            key: string;
            value: number;
        }[];
        tokenAttributeBlackboard: Record<
            string,
            {
                key: string;
                value: number;
            }[]
        >;
    }[];
};

export type AddModuleCandidates = {
    dispalyRangeId: boolean;
    upgradeDescription: string;
    talentIndex: number;
    unlockCondition: {
        phase: number;
        level: number;
    };
    requiredPotentialRank: number;
    prefabKey: string | null;
    name: string;
    description: string | null;
    rangeId: string | null;
    blackboard: {
        key: string;
        value: number;
    }[];
    tokenKey: string | null;
};

export type ModuleCandidates = {
    additionalDescription: string;
    unlockCondition: {
        phase: number;
        level: number;
    };
    requiredPotentialRank: number;
    blackboard: {
        key: string;
        value: number;
    }[];
    overrideDescripton: string | null;
    prefabKey: string | null;
    rangeId: string | null;
};

export enum ModuleTarget {
    TRAIT = "TRAIT",
    TALENT_DATA_ONLY = "TALENT_DATA_ONLY",
    TALENT = "TALENT",
}
