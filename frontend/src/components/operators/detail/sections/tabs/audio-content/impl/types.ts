import type { LangType, PlaceType, VoiceData } from "~/types/api/impl/voice";

export interface VoiceLine {
    id: string;
    title: string;
    text: string;
    data?: VoiceData[];
    languages?: LangType[];
    placeType?: PlaceType;
}

export interface VoiceCategory {
    id: string;
    name: string;
    lines: VoiceLine[];
}
