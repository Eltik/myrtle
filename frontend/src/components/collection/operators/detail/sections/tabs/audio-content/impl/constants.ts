import type { LangType } from "~/types/api/impl/voice";

// Category display order
export const CATEGORY_ORDER = ["Greetings", "Combat", "Interaction", "Base", "Level Up", "Recruitment", "Squad", "Other"];

// Map backend LangType to display labels
export const LANGUAGE_LABELS: Record<LangType, string> = {
    JP: "Japanese",
    CN_MANDARIN: "Chinese",
    EN: "English",
    KR: "Korean",
    CN_TOPOLECT: "Chinese (Regional)",
    GER: "German",
    ITA: "Italian",
    RUS: "Russian",
    FRE: "French",
    LINKAGE: "Collaboration",
};
