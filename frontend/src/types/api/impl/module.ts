// Module types

import type { ItemType } from "./material";

export type ModuleType = "INITIAL" | "ADVANCED";

export type ModuleTarget = "TRAIT" | "TRAIT_DATA_ONLY" | "TALENT_DATA_ONLY" | "TALENT" | "DISPLAY" | "OVERWRITE_BATTLE_DATA" | string; // For unknown types

export interface ModuleItemCost {
    id: string;
    count: number;
    type: ItemType;
}

export interface SubProfession {
    subProfessionId: string;
    subProfessionName: string;
    subProfessionCategory: number;
}

export interface EquipTrackItem {
    charId: string;
    equipId: string;
}

export interface EquipTrack {
    timestamp: number;
    trackList: EquipTrackItem[];
}

export interface Mission {
    template: string;
    desc: string;
    paramList: string[];
    uniEquipMissionId: string;
    uniEquipMissionSort: number;
    uniEquipId: string;
    jumpStageId: string | null;
}

export interface ModuleBlackboard {
    key: string;
    value: number;
}

export interface ModuleUnlockCondition {
    phase: string;
    level: number;
}

export interface AddModuleCandidates {
    displayRangeId: boolean;
    upgradeDescription: string;
    talentIndex: number;
    unlockCondition: ModuleUnlockCondition;
    requiredPotentialRank: number;
    prefabKey: string | null;
    name: string;
    description: string | null;
    rangeId: string | null;
    blackboard: ModuleBlackboard[];
    tokenKey: string | null;
    isHideTalent: boolean | null;
}

export interface ModuleCandidates {
    additionalDescription: string;
    unlockCondition: ModuleUnlockCondition;
    requiredPotentialRank: number;
    blackboard: ModuleBlackboard[];
    overrideDescription: string | null;
    prefabKey: string | null;
    rangeId: string | null;
}

export interface AddOrOverrideTalentDataBundle {
    candidates: AddModuleCandidates[] | null;
}

export interface OverrideTraitDataBundle {
    candidates: ModuleCandidates[] | null;
}

export interface ModulePart {
    resKey: string | null;
    target: ModuleTarget;
    isToken: boolean;
    addOrOverrideTalentDataBundle: AddOrOverrideTalentDataBundle;
    overrideTraitDataBundle: OverrideTraitDataBundle;
}

export interface ModulePhase {
    equipLevel: number;
    parts: ModulePart[];
    attributeBlackboard: ModuleBlackboard[];
    tokenAttributeBlackboard: unknown[];
}

export interface ModuleData {
    phases: ModulePhase[];
}

export interface RawModule {
    uniEquipId: string;
    uniEquipName: string;
    uniEquipIcon: string;
    uniEquipDesc: string;
    typeIcon: string;
    typeName1: string;
    typeName2: string | null;
    equipShiningColor: string;
    showEvolvePhase: string;
    unlockEvolvePhase: string;
    charId: string;
    tmplId: string | null;
    showLevel: number;
    unlockLevel: number;
    unlockFavorPoint: number;
    missionList: string[];
    itemCost: unknown[] | null;
    type: ModuleType;
    uniEquipGetTime: number;
    charEquipOrder: number;
}

export interface Module {
    id: string | null;
    uniEquipId: string;
    uniEquipName: string;
    uniEquipIcon: string;
    image: string | null;
    uniEquipDesc: string;
    typeIcon: string;
    typeName1: string;
    typeName2: string | null;
    equipShiningColor: string;
    showEvolvePhase: string;
    unlockEvolvePhase: string;
    charId: string;
    tmplId: string | null;
    showLevel: number;
    unlockLevel: number;
    unlockFavorPoint: number;
    missionList: string[];
    itemCost: Record<string, ModuleItemCost> | null;
    type: ModuleType;
    uniEquipGetTime: number;
    charEquipOrder: number;
}

export interface Modules {
    equipDict: Record<string, Module>;
    missionList: Record<string, Mission>;
    subProfDict: Record<string, SubProfession>;
    charEquip: Record<string, string[]>;
    equipTrackDict: Record<string, EquipTrack>;
    battleEquip?: BattleEquip;
}

export interface RawModules {
    equipDict: Record<string, RawModule>;
    missionList: Record<string, Mission>;
    subProfDict: Record<string, SubProfession>;
    charEquip: Record<string, string[]>;
    equipTrackDict: unknown[];
}

export type BattleEquip = Record<string, ModuleData>;

export interface UniequipTableFile {
    equipDict: Record<string, RawModule>;
    missionList: Record<string, Mission>;
    subProfDict: Record<string, SubProfession>;
    charEquip: Record<string, string[]>;
    equipTrackDict: unknown[];
}

export interface BattleEquipTableFile {
    equips: Record<string, ModuleData>;
}
