export type Voices = {
    charWords: Record<string, Voice>;
    charExtraWords: Record<
        string,
        {
            wordKey: string;
            charId: string;
            voiceId: string;
            voiceText: string;
        }
    >; // Theresa words :sob:
    voiceLangDict: Record<string, VoiceLang>;
    defaultLangType: string;
    newTagList: string[];
    voiceLangTypeDict: Record<
        LangType,
        {
            name: string;
            groupType: string;
        }
    >;
    voiceLangGroupTypeDict: Record<
        string,
        {
            name: string;
            members: LangType[];
        }
    >;
    charDefaultTypeDict: Record<string, LangType>;
    startTimeWithTypeDict: Record<
        LangType,
        {
            timestamp: number;
            charSet: string[];
        }[]
    >; // Does not include CN_MANDARIN
    displayGroupTypeList: string[];
    displayTypeList: LangType[];
    playVoiceRange: PlaceType;
    fesVoiceData: Record<
        PlaceType,
        {
            showType: PlaceType;
            timeData: {
                timeType: string;
                interval: {
                    startTs: number;
                    endTs: number;
                };
            }[];
        }
    >;
    fesVoiceWeight: Record<
        PlaceType,
        {
            showType: PlaceType;
            weight: number;
        }
    >; // Idk what the point of this is. All the weights are 50 lol
    extraVoiceConfigData: Record<
        string,
        {
            voiceId: string;
            validVoiceLang: LangType[];
        }
    >;
};

export type Voice = {
    charWordId: string;
    wordKey: string;
    charId: string;
    voiceId: string;
    voiceText: string;
    voiceTitle: string;
    voiceIndex: number;
    voiceType: "ENUM" | "ONLY_TEXT";
    unlockType: "DIRECT" | "FAVOR" | "AWAKE";
    unlockParam: {
        valueStr: string | null;
        valueInt: number | null;
    }[];
    lockDescription: string | null;
    placeType: PlaceType;
    voiceAsset: string;

    // Added fields
    id?: string;
    data?: {
        voiceURL?: string;
        language?: LangType;
    }[];
    languages?: LangType[];
};

export enum PlaceType {
    HOME_PLACE = "HOME_PLACE",
    NEW_YEAR = "NEW_YEAR",
    GREETING = "GREETING",
    ANNIVERSARY = "ANNIVERSARY",
    HOME_SHOW = "HOME_SHOW",
    HOME_WAIT = "HOME_WAIT",
    GACHA = "GACHA",
    LEVEL_UP = "LEVEL_UP",
    SQUAD = "SQUAD",
    SQUAD_FIRST = "SQUAD_FIRST",
    BATTLE_START = "BATTLE_START",
    BATTLE_FACE_ENEMY = "BATTLE_FACE_ENEMY",
    BATTLE_SELECT = "BATTLE_SELECT",
    BATTLE_PLACE = "BATTLE_PLACE",
    BATTLE_SKILL_1 = "BATTLE_SKILL_1",
    BATTLE_SKILL_2 = "BATTLE_SKILL_2",
    BATTLE_SKILL_3 = "BATTLE_SKILL_3",
    BATTLE_SKILL_4 = "BATTLE_SKILL_4",
    FOUR_STAR = "FOUR_STAR",
    THREE_STAR = "THREE_STAR",
    TWO_STAR = "TWO_STAR",
    LOSE = "LOSE",
    BUILDING_PLACE = "BUILDING_PLACE",
    BUILDING_TOUCHING = "BUILDING_TOUCHING",
    BUILDING_FAVOR_BUBBLE = "BUILDING_FAVOR_BUBBLE",
    LOADING_PANEL = "LOADING_PANEL",
}

export type VoiceLang = {
    wordkeys: string[];
    charId: string;
    dict: Record<
        string,
        {
            wordkey: string;
            voiceLangType: LangType;
            cvName: string[];
            voicePath: string | null;
        }
    >;
};

export enum LangType {
    CN_MANDARIN,
    JP,
    KR,
    EN,
    RUS,
    ITA,
    CN_TOPOLECT,
    LINKAGE,
    GER,
}
