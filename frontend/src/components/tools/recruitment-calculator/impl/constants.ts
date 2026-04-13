import type { TagType } from "~/types/frontend/impl/tools/recruitment";

// Maximum number of tags that can be selected
export const MAX_SELECTED_TAGS = 5;

// Tag ID to type mapping
// The game's TagGroup field equals TagId, so we must categorize by TagId
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
