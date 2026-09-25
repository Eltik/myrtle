import { BookOpen, Info, Shirt, Sparkles, TrendingUp, Volume2 } from "lucide-react";
import type { messages as detailConstantsMessages } from "#/components/operators/detail/impl/constants.messages";
import type { AudioCategory, OperatorRarityTier } from "#/types/operators";

export type TabType = "info" | "skills" | "levelup" | "skins" | "audio" | "lore";

/** A key in `constants.messages.ts`; resolved by whichever component renders it. */
export type DetailMessageKey = keyof typeof detailConstantsMessages & string;

export const TABS: { type: TabType; labelKey: DetailMessageKey; icon: React.ElementType }[] = [
    { type: "info", labelKey: "tab.info", icon: Info },
    { type: "skills", labelKey: "tab.skills", icon: Sparkles },
    { type: "levelup", labelKey: "tab.levelup", icon: TrendingUp },
    { type: "skins", labelKey: "tab.skins", icon: Shirt },
    { type: "audio", labelKey: "tab.audio", icon: Volume2 },
    { type: "lore", labelKey: "tab.lore", icon: BookOpen },
];

export const RARITY_COLORS: Record<OperatorRarityTier, string> = {
    TIER_6: "text-[var(--rarity-6)] border-[var(--rarity-6)]/50",
    TIER_5: "text-[var(--rarity-5)] border-[var(--rarity-5)]/50",
    TIER_4: "text-[var(--rarity-4)] border-[var(--rarity-4)]/50",
    TIER_3: "text-[var(--rarity-3)] border-[var(--rarity-3)]/50",
    TIER_2: "text-[var(--rarity-2)] border-[var(--rarity-2)]/50",
    TIER_1: "text-[var(--rarity-1)] border-[var(--rarity-1)]/50",
};

export const RARITY_GLOW: Record<OperatorRarityTier, string> = {
    TIER_6: "drop-shadow-[0_0_20px_rgba(255,127,39,0.5)]",
    TIER_5: "drop-shadow-[0_0_20px_rgba(247,213,76,0.4)]",
    TIER_4: "drop-shadow-[0_0_15px_rgba(201,184,240,0.4)]",
    TIER_3: "drop-shadow-[0_0_15px_rgba(125,211,252,0.4)]",
    TIER_2: "drop-shadow-[0_0_15px_rgba(134,239,172,0.3)]",
    TIER_1: "drop-shadow-[0_0_10px_rgba(255,255,255,0.2)]",
};

export const VOICE_LANGUAGE_LABEL_KEY: Record<string, DetailMessageKey> = {
    JP: "voice.lang.jp",
    CN_MANDARIN: "voice.lang.cnMandarin",
    EN: "voice.lang.en",
    KR: "voice.lang.kr",
    CN_TOPOLECT: "voice.lang.cnTopolect",
    GER: "voice.lang.ger",
    ITA: "voice.lang.ita",
    RUS: "voice.lang.rus",
    FRE: "voice.lang.fre",
    SPA: "voice.lang.spa",
    LINKAGE: "voice.lang.linkage",
};

export const VOICE_LANGUAGE_ORDER = ["JP", "CN_MANDARIN", "EN", "KR", "CN_TOPOLECT", "GER", "ITA", "RUS", "FRE", "SPA", "LINKAGE"] as const;

export const VOICE_LANGUAGE_SHORT_KEY: Record<string, DetailMessageKey> = {
    JP: "voice.lang.short.jp",
    CN_MANDARIN: "voice.lang.short.cnMandarin",
    EN: "voice.lang.short.en",
    KR: "voice.lang.short.kr",
    CN_TOPOLECT: "voice.lang.short.cnTopolect",
    GER: "voice.lang.short.ger",
    ITA: "voice.lang.short.ita",
    RUS: "voice.lang.short.rus",
    FRE: "voice.lang.short.fre",
    SPA: "voice.lang.short.spa",
    LINKAGE: "voice.lang.short.linkage",
};

/**
 * Voice-line buckets in display order. These are IDENTIFIERS, not labels: the
 * tab ids are derived from them and `VOICE_CATEGORY_MAP` sorts the game's raw
 * `placeType` tokens into them. The text a reader sees comes from
 * `VOICE_CATEGORY_LABEL_KEY`.
 */
export const VOICE_CATEGORY_ORDER = ["Greetings", "Conversations", "Trust", "Promotions", "Battle", "Idle", "Dorm", "Special", "Other"];

export const VOICE_CATEGORY_LABEL_KEY: Record<string, DetailMessageKey> = {
    Greetings: "voice.category.greetings",
    Conversations: "voice.category.conversations",
    Trust: "voice.category.trust",
    Promotions: "voice.category.promotions",
    Battle: "voice.category.battle",
    Idle: "voice.category.idle",
    Dorm: "voice.category.dorm",
    Special: "voice.category.special",
    Other: "voice.category.other",
};

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

export const SFX_CATEGORY_ORDER: AudioCategory[] = ["deploy", "attack", "skill", "voice", "other"];

export const SFX_CATEGORY_LABEL_KEY: Record<AudioCategory, DetailMessageKey> = {
    deploy: "sfx.category.deploy",
    attack: "sfx.category.attack",
    skill: "sfx.category.skill",
    voice: "sfx.category.voice",
    other: "sfx.category.other",
};

export const SFX_EVENT_LABEL_KEY: Record<string, DetailMessageKey> = {
    ON_UNIT_BORN: "sfx.event.unitBorn",
    ON_ABILITY_START: "sfx.event.abilityStart",
    ON_ABILITY_HIT: "sfx.event.abilityHit",
    ON_ABILITY_ON: "sfx.event.abilityOn",
    ON_ABILITY_END: "sfx.event.abilityEnd",
    ON_ABILITY_ATTACK_FINISH: "sfx.event.abilityAttackFinish",
    ON_ABILITY_CHECK_POINT: "sfx.event.abilityCheckPoint",
    ON_SKILL_START: "sfx.event.skillStart",
    ON_SKILL_CHANT_START: "sfx.event.skillChantStart",
    ON_SKILL_SPECIAL_POINT: "sfx.event.skillSpecialPoint",
    ON_SKILL_FINISH: "sfx.event.skillFinish",
    ON_SKILL_ON: "sfx.event.skillOn",
    ON_SKILL_FAILED: "sfx.event.skillFailed",
    ON_UNIT_DEAD: "sfx.event.unitDead",
    ON_SPINE_EVENT_TRIGGER: "sfx.event.spineEventTrigger",
    ON_CUSTOM_TRIGGER: "sfx.event.customTrigger",
};
