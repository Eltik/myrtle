import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

/**
 * The label tables in `constants.ts` are plain data in a module with no React,
 * so they carry message KEYS and whichever component renders a row resolves it
 * with `t()`.
 *
 * These are this site's own short labels for backend enum values
 * (`stageType`, `difficulty`, `dropType`, `occPercent`), not game vocabulary:
 * the stage names, codes and item names they sit beside come from the
 * game-data layer and are deliberately absent.
 */
export const namespace = "stages";

export const messages = {
    "stageType.MAIN": {
        text: "Main Theme",
        description: "Stage-type label under the stage title: a mainline campaign operation.",
    },
    "stageType.SUB": {
        text: "Sub Stage",
        description: "Stage-type label: a secondary operation inside a chapter.",
    },
    "stageType.ACTIVITY": {
        text: "Event",
        description: "Stage-type label: an operation belonging to a limited-time event.",
    },
    "stageType.DAILY": {
        text: "Daily / Resource",
        description: "Stage-type label: a rotating daily resource-farming operation.",
    },
    "stageType.CAMPAIGN": {
        text: "Campaign",
        description: "Stage-type label for the game's Campaign (Annihilation) stage type.",
    },
    "stageType.CLIMB_TOWER": {
        text: "Stationary Security",
        description: "Stage-type label for the Stationary Security Service game mode; an in-game mode name.",
    },
    "stageType.GUIDE": {
        text: "Tutorial",
        description: "Stage-type label: a tutorial operation.",
    },
    "stageType.SPECIAL_STORY": {
        text: "Special Story",
        description: "Stage-type label: a special story operation.",
    },
    "stageType.UNKNOWN": {
        text: "Unknown",
        description: "Stage-type label when the game data ships a stage type this site does not name.",
    },
    "difficulty.NORMAL": {
        text: "Normal",
        description: "Stage difficulty: the standard version of an operation.",
    },
    "difficulty.FOUR_STAR": {
        text: "Challenge Mode",
        description: "Stage difficulty badge for the game's Challenge Mode version of an operation.",
    },
    "difficulty.SIX_STAR": {
        text: "Extreme",
        description: "Stage difficulty badge for the hardest version of an operation.",
    },
    "difficulty.UNKNOWN": {
        text: "Unknown",
        description: "Stage difficulty badge when the game data ships a difficulty this site does not name.",
    },
    "dropType.ONCE": {
        text: "First Clear",
        description: "Heading over the drops awarded only the first time a stage is cleared.",
    },
    "dropType.COMPLETE": {
        text: "On Completion",
        description: "Heading over the drops awarded for completing a stage.",
    },
    "dropType.NORMAL": {
        text: "Regular Drops",
        description: "Heading over a stage's ordinary drop pool.",
    },
    "dropType.SPECIAL": {
        text: "Special Drops",
        description: "Heading over a stage's special drop pool.",
    },
    "dropType.ADDITIONAL": {
        text: "Bonus Drops",
        description: "Heading over a stage's extra drop pool.",
    },
    "dropType.CONDITION_DROP": {
        text: "Conditional Drops",
        description: "Heading over drops that need a condition met to appear.",
    },
    "occ.ALWAYS": {
        text: "Guaranteed",
        description: "Drop-rate tier, the highest: the item always drops. Shown as a short uppercase label beside a 5-bar meter.",
    },
    "occ.ALMOST": {
        text: "Common",
        description: "Drop-rate tier, second highest. Short uppercase label beside a 5-bar meter.",
    },
    "occ.USUAL": {
        text: "Uncommon",
        description: "Drop-rate tier, middle. Short uppercase label beside a 5-bar meter.",
    },
    "occ.OFTEN": {
        text: "Rare",
        description: "Drop-rate tier, fourth. Short uppercase label beside a 5-bar meter. Despite the game's enum name, this is a LOW rate.",
    },
    "occ.SOMETIMES": {
        text: "Very Rare",
        description: "Drop-rate tier, fifth. Short uppercase label beside a 5-bar meter.",
    },
    "occ.RARELY": {
        text: "Very Rare",
        description: "Drop-rate tier, the lowest. Same English as the tier above it, but a separate key so a language that distinguishes them can.",
    },
    "occ.FALLBACK": {
        text: "Drops",
        description: "Drop-rate label when the game data ships a rate this site does not name.",
    },
} satisfies MessageMap;

// `dynamic`: these keys are stored on a constants entry and resolved by the
// consuming component as `t(item.labelKey)`, so the extractor has no literal
// call site to match them against.
export const { keys } = defineMessages({ namespace, messages, dynamic: true });
