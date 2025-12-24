import type { TagType } from "./types";

// Maximum number of tags that can be selected
export const MAX_SELECTED_TAGS = 5;

// Tag group ID to type mapping (from backend tagGroup field)
// Group 0: Qualification (Top Operator, Senior Operator, Starter, Robot)
// Group 1: Position (Melee, Ranged)
// Group 2: Class (Guard, Sniper, etc.)
// Group 3: Affix (Healing, DPS, etc.)
export const TAG_GROUP_MAP: Record<number, TagType> = {
    0: "qualification",
    1: "position",
    2: "class",
    3: "affix",
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
