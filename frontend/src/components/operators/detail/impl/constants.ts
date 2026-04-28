import { BookOpen, Info, Shirt, Sparkles, TrendingUp, Volume2 } from "lucide-react";
import type { OperatorRarityTier } from "#/types/operators";

export type TabType = "info" | "skills" | "levelup" | "skins" | "audio" | "lore";

export const TABS: { type: TabType; label: string; icon: React.ElementType }[] = [
    { type: "info", label: "Information", icon: Info },
    { type: "skills", label: "Skills", icon: Sparkles },
    { type: "levelup", label: "Level-Up Cost", icon: TrendingUp },
    { type: "skins", label: "Skins", icon: Shirt },
    { type: "audio", label: "Audio/SFX", icon: Volume2 },
    { type: "lore", label: "Lore", icon: BookOpen },
];

export const RARITY_COLORS: Record<OperatorRarityTier, string> = {
    TIER_6: "text-[#f7a452] border-[#f7a452]/50",
    TIER_5: "text-[#f7e79e] border-[#f7e79e]/50",
    TIER_4: "text-[#bcabdb] border-[#bcabdb]/50",
    TIER_3: "text-[#88c8e3] border-[#88c8e3]/50",
    TIER_2: "text-[#7ef2a3] border-[#7ef2a3]/50",
    TIER_1: "text-white border-white/50",
};

export const RARITY_GLOW: Record<OperatorRarityTier, string> = {
    TIER_6: "drop-shadow-[0_0_20px_rgba(255,154,74,0.5)]",
    TIER_5: "drop-shadow-[0_0_20px_rgba(255,230,109,0.4)]",
    TIER_4: "drop-shadow-[0_0_15px_rgba(201,184,240,0.4)]",
    TIER_3: "drop-shadow-[0_0_15px_rgba(125,211,252,0.4)]",
    TIER_2: "drop-shadow-[0_0_15px_rgba(134,239,172,0.3)]",
    TIER_1: "drop-shadow-[0_0_10px_rgba(255,255,255,0.2)]",
};

export const VOICE_LANGUAGE_LABELS: Record<string, string> = {
    JP: "Japanese",
    CN_MANDARIN: "Mandarin",
    EN: "English",
    KR: "Korean",
    CN_TOPOLECT: "Cantonese",
    GER: "German",
    ITA: "Italian",
    RUS: "Russian",
    FRE: "French",
    SPA: "Spanish",
    LINKAGE: "Linkage",
};

export const VOICE_LANGUAGE_ORDER = ["JP", "CN_MANDARIN", "EN", "KR", "CN_TOPOLECT", "GER", "ITA", "RUS", "FRE", "SPA", "LINKAGE"] as const;

export const VOICE_CATEGORY_ORDER = ["Greetings", "Conversations", "Trust", "Promotions", "Battle", "Idle", "Dorm", "Special", "Other"];

export const VOICE_CATEGORY_MAP: Record<string, string> = {
    HOME_PLACE: "Greetings",
    HOME_SHOW: "Greetings",
    HOME_WAIT: "Greetings",
    GREETING: "Greetings",
    BIRTHDAY: "Special",
    NEW_YEAR: "Special",
    ANNIVERSARY: "Special",
    GACHA: "Special",
    SQUAD: "Conversations",
    SQUAD_FIRST: "Conversations",
    LEVEL_UP: "Promotions",
    EVOLVE_ONE: "Promotions",
    EVOLVE_TWO: "Promotions",
    BATTLE_START: "Battle",
    BATTLE_FACE_ENEMY: "Battle",
    BATTLE_SELECT: "Battle",
    BATTLE_PLACE: "Battle",
    BATTLE_SKILL_1: "Battle",
    BATTLE_SKILL_2: "Battle",
    BATTLE_SKILL_3: "Battle",
    BATTLE_SKILL_4: "Battle",
    FOUR_STAR: "Battle",
    THREE_STAR: "Battle",
    TWO_STAR: "Battle",
    LOSE: "Battle",
    BUILDING_PLACE: "Dorm",
    BUILDING_TOUCHING: "Dorm",
    BUILDING_FAVOR_BUBBLE: "Dorm",
    LOADING_PANEL: "Other",
};
