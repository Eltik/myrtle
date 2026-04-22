export type LangType = "CN_MANDARIN" | "JP" | "KR" | "EN" | "RUS" | "ITA" | "CN_TOPOLECT" | "LINKAGE" | "GER" | "FRE" | "SPA";

export type VoiceType = "ENUM" | "ONLY_TEXT";

export type UnlockType = "DIRECT" | "FAVOR" | "AWAKE";

export type PlaceType =
    | "HOME_PLACE"
    | "NEW_YEAR"
    | "GREETING"
    | "ANNIVERSARY"
    | "BIRTHDAY"
    | "HOME_SHOW"
    | "HOME_WAIT"
    | "GACHA"
    | "LEVEL_UP"
    | "EVOLVE_ONE"
    | "EVOLVE_TWO"
    | "SQUAD"
    | "SQUAD_FIRST"
    | "BATTLE_START"
    | "BATTLE_FACE_ENEMY"
    | "BATTLE_SELECT"
    | "BATTLE_PLACE"
    | "BATTLE_SKILL_1"
    | "BATTLE_SKILL_2"
    | "BATTLE_SKILL_3"
    | "BATTLE_SKILL_4"
    | "FOUR_STAR"
    | "THREE_STAR"
    | "TWO_STAR"
    | "LOSE"
    | "BUILDING_PLACE"
    | "BUILDING_TOUCHING"
    | "BUILDING_FAVOR_BUBBLE"
    | "LOADING_PANEL";

export interface IUnlockParam {
    valueStr: string | null;
    valueInt: number | null;
}

export interface IVoiceData {
    voiceUrl: string | null;
    language: LangType | null;
    cvName: string[] | null;
}

export interface ICharExtraWord {
    wordKey: string;
    charId: string;
    voiceId: string;
    voiceText: string;
}

export interface IVoiceLangTypeInfo {
    name: string;
    groupType: string;
}

export interface IVoiceLangGroupType {
    name: string;
    members: LangType[];
}

export interface IStartTimeWithType {
    timestamp: number;
    charSet: string[];
}

export interface IFesTimeInterval {
    startTs: number;
    endTs: number;
}

export interface IFesTimeData {
    timeType: string;
    interval: IFesTimeInterval;
}

export interface IFesVoiceData {
    showType: PlaceType;
    timeData: IFesTimeData[];
}

export interface IFesVoiceWeight {
    showType: PlaceType;
    weight: number;
}

export interface IExtraVoiceConfigData {
    voiceId: string;
    validVoiceLang: LangType[];
}

export interface IVoiceLangDictEntry {
    wordkey: string;
    voiceLangType: LangType;
    cvName: string[];
    voicePath: string | null;
}

export interface IVoiceLang {
    wordkeys: string[];
    charId: string;
    /** Keyed by `LangType` (e.g. "JP", "EN") — each entry lists the CV names for that language. */
    dict: Record<string, IVoiceLangDictEntry>;
}

export interface IVoice {
    charWordId: string;
    wordKey: string;
    charId: string;
    voiceId: string;
    voiceText: string;
    voiceTitle: string;
    voiceIndex: number;
    voiceType: VoiceType;
    unlockType: UnlockType;
    unlockParam: IUnlockParam[];
    lockDescription: string | null;
    placeType: PlaceType;
    voiceAsset: string;
    id: string | null;
    data: IVoiceData[] | null;
    languages: LangType[] | null;
}

export interface IVoices {
    charWords: Record<string, IVoice>;
    charExtraWords: Record<string, ICharExtraWord>;
    /** Primary lookup for "voice actors of operator X" — keyed by operator/char id. */
    voiceLangDict: Record<string, IVoiceLang>;
    defaultLangType: string;
    newTagList: string[];
    voiceLangTypeDict: Record<string, IVoiceLangTypeInfo>;
    voiceLangGroupTypeDict: Record<string, IVoiceLangGroupType>;
    charDefaultTypeDict: Record<string, LangType>;
    startTimeWithTypeDict: Record<string, IStartTimeWithType[]>;
    displayGroupTypeList: string[];
    displayTypeList: LangType[];
    playVoiceRange: PlaceType;
    fesVoiceData: Record<string, IFesVoiceData>;
    fesVoiceWeight: Record<string, IFesVoiceWeight>;
    extraVoiceConfigData: Record<string, IExtraVoiceConfigData>;
}
