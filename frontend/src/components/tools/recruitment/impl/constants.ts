import type { TagType } from "./types";

// Maximum number of tags that can be selected
export const MAX_SELECTED_TAGS = 5;

// Tag ID to type mapping
// Position: Melee (9), Ranged (10)
// Class: Guard (1), Sniper (2), Defender (3), Medic (4), Supporter (5), Caster (6), Specialist (7), Vanguard (8)
// Qualification: Top Operator (11), Senior Operator (14), Starter (17), Robot (28)
// Affix: Everything else
export const TAG_ID_TO_TYPE_MAP: Record<number, TagType> = {
    // Class tags (1-8)
    1: "class", // Guard
    2: "class", // Sniper
    3: "class", // Defender
    4: "class", // Medic
    5: "class", // Supporter
    6: "class", // Caster
    7: "class", // Specialist
    8: "class", // Vanguard
    // Position tags (9-10)
    9: "position", // Melee
    10: "position", // Ranged
    // Qualification tags
    11: "qualification", // Top Operator
    14: "qualification", // Senior Operator
    17: "qualification", // Starter
    28: "qualification", // Robot
};

// Display order for tag groups
export const TAG_GROUP_ORDER: TagType[] = ["qualification", "position", "class", "affix"];

// Tag group display labels
export const TAG_GROUP_LABELS: Record<TagType, string> = {
    qualification: "Qualification",
    position: "Position",
    class: "Class",
    affix: "Affix",
};

// Special tag IDs for guaranteed rarity calculations
export const TOP_OPERATOR_TAG_ID = 11;
export const SENIOR_OPERATOR_TAG_ID = 14;
export const ROBOT_TAG_ID = 28;

// Rarity star colors for display
export const RARITY_COLORS: Record<number, { bg: string; text: string; border: string }> = {
    6: { bg: "bg-orange-500/15", text: "text-orange-600 dark:text-orange-400", border: "border-orange-500/30" },
    5: { bg: "bg-amber-500/15", text: "text-amber-600 dark:text-amber-400", border: "border-amber-500/30" },
    4: { bg: "bg-purple-500/15", text: "text-purple-600 dark:text-purple-400", border: "border-purple-500/30" },
    3: { bg: "bg-blue-500/15", text: "text-blue-600 dark:text-blue-400", border: "border-blue-500/30" },
    2: { bg: "bg-green-500/15", text: "text-green-600 dark:text-green-400", border: "border-green-500/30" },
    1: { bg: "bg-zinc-500/15", text: "text-zinc-600 dark:text-zinc-400", border: "border-zinc-500/30" },
};

// Profession display names
export const PROFESSION_LABELS: Record<string, string> = {
    WARRIOR: "Guard",
    SNIPER: "Sniper",
    TANK: "Defender",
    MEDIC: "Medic",
    SUPPORT: "Supporter",
    CASTER: "Caster",
    SPECIAL: "Specialist",
    PIONEER: "Vanguard",
};
