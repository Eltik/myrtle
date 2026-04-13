// Handbook types

import type { ItemType } from "./material";

export type OperatorGender =
    | "Unknown"
    | "Female"
    | "Male"
    | "Male]" // Arene bug
    | "Conviction";

export type OperatorBirthPlace =
    | "Unknown"
    | "Undisclosed"
    | "Higashi"
    | "Kazimierz"
    | "Vouivre"
    | "Laterano"
    | "Victoria"
    | "Rim Billiton"
    | "Leithanien"
    | "Bolívar"
    | "Sargon"
    | "Kjerag"
    | "Columbia"
    | "Sami"
    | "Iberia"
    | "Kazdel"
    | "Minos"
    | "Lungmen"
    | "Siracusa"
    | "Yan"
    | "Ursus"
    | "Siesta"
    | "RIM Billiton"
    | "Ægir"
    | "Durin"
    | "Siesta (Independent City)"
    | "Ægir Region"
    | "Unknown as requested by management agency"
    | "Rhodes Island"
    | "Far East";

export type OperatorRace =
    | "Undisclosed"
    | "Zalak"
    | "Oni"
    | "Savra"
    | "Durin"
    | "Kuranta"
    | "Vouivre"
    | "Liberi"
    | "Feline"
    | "Cautus"
    | "Perro"
    | "Reproba"
    | "Sankta"
    | "Sarkaz"
    | "Vulpo"
    | "Elafia"
    | "Phidia"
    | "Ægir"
    | "Anaty"
    | "Itra"
    | "Unknown (Suspected Liberi)"
    | "Archosauria"
    | "Unknown"
    | "Lupo"
    | "Forte"
    | "Ursus"
    | "Petram"
    | "Cerato"
    | "Caprinae"
    | "Draco"
    | "Anura"
    | "Anasa"
    | "Cautus/Chimera"
    | "Kylin"
    | "Pilosa"
    | "Unknown as requested by management agency"
    | "Manticore"
    | "Lung"
    | "Aslan"
    | "Elf"
    | "Sa■&K?uSxw?"; // Corrupted text

export interface BasicInfo {
    codeName: string;
    gender: OperatorGender;
    combatExperience: string;
    placeOfBirth: OperatorBirthPlace;
    dateOfBirth: string;
    race: OperatorRace;
    height: string;
    infectionStatus: string;
}

export interface PhysicalExam {
    physicalStrength: string;
    mobility: string;
    physicalResilience: string;
    tacticalAcumen: string;
    combatSkill: string;
    originiumArtsAssimilation: string;
}

export interface OperatorProfile {
    basicInfo: BasicInfo;
    physicalExam: PhysicalExam;
}

export interface HandbookRewardItem {
    id: string;
    count: number;
    type: ItemType;
}

export interface TeamMission {
    id: string;
    sort: number;
    powerId: string;
    powerName: string;
    item: HandbookRewardItem;
    favorPoint: number;
}

export interface HandbookDisplayCondition {
    charId: string;
    conditionCharId: string;
    type: string;
}

export interface HandbookStageTime {
    timestamp: number;
    charSet: string[];
}

export interface HandbookStory {
    storyText: string;
    unlockType: string;
    unLockParam: string;
    unLockString: string;
    patchIdList: string[] | null;
}

export interface HandbookStoryTextAudio {
    stories: HandbookStory[];
    storyTitle: string;
    unLockorNot: boolean;
}

export interface HandbookUnlockParam {
    unlockType: string;
    unlockParam1: string | null;
    unlockParam2: string | null;
    unlockParam3: string | null;
}

export interface HandbookAvgEntry {
    storyId: string;
    storySetId: string;
    storySort: number;
    storyCanShow: boolean;
    storyIntro: string;
    storyInfo: string;
    storyTxt: string;
}

export interface HandbookAvgList {
    storySetId: string;
    storySetName: string;
    sortId: number;
    storyGetTime: number;
    rewardItem: HandbookRewardItem[];
    unlockParam: HandbookUnlockParam[];
    avgList: HandbookAvgEntry[];
    charId: string;
}

export interface HandbookItem {
    charID: string;
    infoName: string;
    isLimited: boolean;
    storyTextAudio: HandbookStoryTextAudio[];
    handbookAvgList: HandbookAvgList[];
}

export interface NPCUnlockInfo {
    unLockType: string;
    unLockParam: string;
    unLockString: string | null;
}

export interface HandbookNPCItem {
    npcId: string;
    name: string;
    appellation: string;
    profession: string;
    illustList: string[] | null;
    designerList: string[] | null;
    cv: string;
    displayNumber: string;
    nationId: string | null;
    groupId: string | null;
    teamId: string | null;
    resType: string;
    npcShowAudioInfoFlag: boolean;
    unlockDict: Record<string, NPCUnlockInfo>;
}

export interface HandbookStageData {
    charId: string;
    stageId: string;
    levelId: string;
    zoneId: string;
    code: string;
    name: string;
    loadingPicId: string;
    description: string;
    unlockParam: HandbookUnlockParam[];
    rewardItem: HandbookRewardItem[];
    stageNameForShow: string;
    zoneNameForShow: string;
    picId: string;
    stageGetTime: number;
}

export interface Handbook {
    handbookDict: Record<string, HandbookItem>;
    npcDict: Record<string, HandbookNPCItem>;
    teamMissionList: Record<string, TeamMission>;
    handbookDisplayConditionList: Record<string, HandbookDisplayCondition>;
    handbookStageData: Record<string, HandbookStageData>;
    handbookStageTime: HandbookStageTime[];
}

export interface HandbookTableFile {
    handbookDict: Record<string, HandbookItem>;
    npcDict: Record<string, HandbookNPCItem>;
    teamMissionList: Record<string, TeamMission>;
    handbookDisplayConditionList: Record<string, HandbookDisplayCondition>;
    handbookStageData: Record<string, HandbookStageData>;
    handbookStageTime: HandbookStageTime[];
}
