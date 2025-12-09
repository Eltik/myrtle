// Voice types

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

export type LangType = "CN_MANDARIN" | "JP" | "KR" | "EN" | "RUS" | "ITA" | "CN_TOPOLECT" | "LINKAGE" | "GER" | "FRE";

export type VoiceType = "ENUM" | "ONLY_TEXT";

export type UnlockType = "DIRECT" | "FAVOR" | "AWAKE";

export interface UnlockParam {
    valueStr: string | null;
    valueInt: number | null;
}

export interface VoiceData {
    voiceUrl: string | null;
    language: LangType | null;
    cvName: string[] | null;
}

export interface CharExtraWord {
    wordKey: string;
    charId: string;
    voiceId: string;
    voiceText: string;
}

export interface VoiceLangTypeInfo {
    name: string;
    groupType: string;
}

export interface VoiceLangGroupType {
    name: string;
    members: LangType[];
}

export interface StartTimeWithType {
    timestamp: number;
    charSet: string[];
}

export interface FesTimeInterval {
    startTs: number;
    endTs: number;
}

export interface FesTimeData {
    timeType: string;
    interval: FesTimeInterval;
}

export interface FesVoiceData {
    showType: PlaceType;
    timeData: FesTimeData[];
}

export interface FesVoiceWeight {
    showType: PlaceType;
    weight: number;
}

export interface ExtraVoiceConfigData {
    voiceId: string;
    validVoiceLang: LangType[];
}

export interface VoiceLangDictEntry {
    wordkey: string;
    voiceLangType: LangType;
    cvName: string[];
    voicePath: string | null;
}

export interface VoiceLang {
    wordkeys: string[];
    charId: string;
    dict: Record<string, VoiceLangDictEntry>;
}

export interface RawVoice {
    charWordId: string;
    wordKey: string;
    charId: string;
    voiceId: string;
    voiceText: string;
    voiceTitle: string;
    voiceIndex: number;
    voiceType: VoiceType;
    unlockType: UnlockType;
    unlockParam: UnlockParam[];
    lockDescription: string | null;
    placeType: PlaceType;
    voiceAsset: string;
}

export interface Voice {
    charWordId: string;
    wordKey: string;
    charId: string;
    voiceId: string;
    voiceText: string;
    voiceTitle: string;
    voiceIndex: number;
    voiceType: VoiceType;
    unlockType: UnlockType;
    unlockParam: UnlockParam[];
    lockDescription: string | null;
    placeType: PlaceType;
    voiceAsset: string;
    id: string | null;
    data: VoiceData[] | null;
    languages: LangType[] | null;
}

export interface Voices {
    charWords: Record<string, Voice>;
    charExtraWords: Record<string, CharExtraWord>;
    voiceLangDict: Record<string, VoiceLang>;
    defaultLangType: string;
    newTagList: string[];
    voiceLangTypeDict: Record<LangType, VoiceLangTypeInfo>;
    voiceLangGroupTypeDict: Record<string, VoiceLangGroupType>;
    charDefaultTypeDict: Record<string, LangType>;
    startTimeWithTypeDict: Record<LangType, StartTimeWithType[]>;
    displayGroupTypeList: string[];
    displayTypeList: LangType[];
    playVoiceRange: PlaceType;
    fesVoiceData: Record<PlaceType, FesVoiceData>;
    fesVoiceWeight: Record<PlaceType, FesVoiceWeight>;
    extraVoiceConfigData: Record<string, ExtraVoiceConfigData>;
}

export interface RawVoices {
    charWords: Record<string, RawVoice>;
    charExtraWords: Record<string, CharExtraWord>;
    voiceLangDict: Record<string, VoiceLang>;
    defaultLangType: string;
    newTagList: string[];
    voiceLangTypeDict: Record<LangType, VoiceLangTypeInfo>;
    voiceLangGroupTypeDict: Record<string, VoiceLangGroupType>;
    charDefaultTypeDict: Record<string, LangType>;
    startTimeWithTypeDict: Record<LangType, StartTimeWithType[]>;
    displayGroupTypeList: string[];
    displayTypeList: LangType[];
    playVoiceRange: PlaceType;
    fesVoiceData: Record<PlaceType, FesVoiceData>;
    fesVoiceWeight: Record<PlaceType, FesVoiceWeight>;
    extraVoiceConfigData: Record<string, ExtraVoiceConfigData>;
}

export interface VoicesTableFile {
    charWords: Record<string, RawVoice>;
    charExtraWords: Record<string, CharExtraWord>;
    voiceLangDict: Record<string, VoiceLang>;
    defaultLangType: string;
    newTagList: string[];
}
