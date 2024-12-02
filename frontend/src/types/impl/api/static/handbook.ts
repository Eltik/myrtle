import type { ItemType } from "./material";

export type OperatorProfile = {
    basicInfo: {
        codeName: string;
        gender: "Unknown" | "Female" | "Male" | "Male]" | "Conviction"; // Arene is bugged and has ] at the end lol
        combatExperience: string;
        placeOfBirth: OperatorBirthPlace;
        dateOfBirth: string;
        race: OperatorRace;
        height: string;
        infectionStatus: string;
    };
    physicalExam: {
        physicalStrength: string;
        mobility: string;
        physicalResilience: string;
        tacticalAcumen: string;
        combatSkill: string;
        originiumArtsAssimilation: string;
    };
};

export enum OperatorBirthPlace {
    Unknown,
    Undisclosed,
    Higashi,
    Kazimierz,
    Vouivre,
    Laterano,
    Victoria,
    "Rim Billiton",
    Leithanien,
    Bolívar,
    Sargon,
    Kjerag,
    Columbia,
    Sami,
    Iberia,
    Kazdel,
    Minos,
    Lungmen,
    Siracusa,
    Yan,
    Ursus,
    Siesta,
    "RIM Billiton",
    Ægir,
    Durin,
    "Siesta (Independent City)",
    "Ægir Region",
    "Unknown as requested by management agency",
    "Rhodes Island",
    "Far East",
}

export enum OperatorRace {
    Undisclosed,
    Zalak,
    Oni,
    Savra,
    Durin,
    Kuranta,
    Vouivre,
    Liberi,
    Feline,
    Cautus,
    Perro,
    Reproba,
    Sankta,
    Sarkaz,
    Vulpo,
    Elafia,
    Phidia,
    Ægir,
    Anaty,
    Itra,
    "Unknown (Suspected Liberi)",
    Archosauria,
    Unknown,
    Lupo,
    Forte,
    Ursus,
    Petram,
    Cerato,
    Caprinae,
    Draco,
    Anura,
    Anasa,
    "Cautus/Chimera",
    Kylin,
    Pilosa,
    "Unknown as requested by management agency",
    Manticore,
    Lung,
    Aslan,
    Elf,
    "Sa■&K?uSxw?",
}

export type Handbook = {
    handbookDict: Record<string, HandbookItem>;
    npcDict: Record<string, HandbookNPCItem>;
    teamMissionList: Record<
        string,
        {
            id: string;
            sort: number;
            powerId: string;
            powerName: string;
            item: {
                id: string;
                count: number;
                type: ItemType;
            };
            favorPoint: number;
        }
    >;
    handbookDisplayConditionList: Record<
        string,
        {
            charId: string;
            conditionCharId: string;
            type: string;
        }
    >;
    handbookStageData: Record<string, HandbookStageData>;
    handbookStageTime: {
        timestamp: number;
        charSet: string[];
    }[];
};

export type HandbookItem = {
    charID: string;
    infoName: string;
    isLimited: boolean;
    storyTextAudio: HandbookStoryTextAudio[];
    handbookAvgList: HandbookAvgList[];
};

export type HandbookStoryTextAudio = {
    stories: {
        storyText: string;
        unlockType: string;
        unLockParam: string;
        unLockString: string;
        patchIdList: string[] | null;
    }[];
    storyTitle: string;
    unLockorNot: boolean;
};

export type HandbookAvgList = {
    storySetId: string;
    storySetName: string;
    sortId: number;
    storyGetTime: number;
    rewardItem: {
        id: string;
        count: number;
        type: ItemType;
    }[];
    unlockParam: {
        unlockType: string;
        unlockParam1: string | null;
        unlockParam2: string | null;
        unlockParam3: string | null;
    }[];
    avgList: {
        storyId: string;
        storySetId: string;
        storySort: number;
        storyCanShow: boolean;
        storyIntro: string;
        storyInfo: string;
        storyTxt: string;
    }[];
    charId: string;
};

export type HandbookNPCItem = {
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
    unlockDict: Record<
        string,
        {
            unLockType: string;
            unLockParam: string;
            unLockString: string | null;
        }
    >;
};

export type HandbookStageData = {
    charId: string;
    stageId: string;
    levelId: string;
    zoneId: string;
    code: string;
    name: string;
    loadingPicId: string;
    description: string;
    unlockParam: {
        unlockType: string;
        unlockParam1: string | null;
        unlockParam2: string | null;
        unlockParam3: string | null;
    }[];
    rewardItem: {
        id: string;
        count: number;
        type: ItemType;
    }[];
    stageNameForShow: string;
    zoneNameForShow: string;
    picId: string;
    stageGetTime: number;
};
