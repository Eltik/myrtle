import type { messages as recruitConstantsMessages } from "./constants.messages";
import type { TagType } from "./types";

/** A key in `constants.messages.ts`; resolved by whichever component renders it. */
export type RecruitMessageKey = keyof typeof recruitConstantsMessages & string;

export const MAX_SELECTED_TAGS = 5;

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

export const TAG_GROUP_ORDER: TagType[] = ["qualification", "position", "class", "affix"];
export const TAG_GROUP_LABEL_KEYS: Record<TagType, RecruitMessageKey> = {
    qualification: "recruit.group.qualification",
    position: "recruit.group.position",
    class: "recruit.group.class",
    affix: "recruit.group.affix",
};

/**
 * The gacha tag id of each profession, as the game's tag list numbers them: ids 1 to 8 are the classes on every
 * server, in this order. The backend names classes from the same numbering (`PROFESSION_TAG_IDS` in
 * `backend/src/core/gamedata/types/gacha.rs`, the source of an operator's `professionName`); change both together.
 */
export const PROFESSION_TAG_ID: Record<string, number> = { WARRIOR: 1, SNIPER: 2, TANK: 3, MEDIC: 4, SUPPORT: 5, CASTER: 6, SPECIAL: 7, PIONEER: 8 };
export const MELEE_TAG_ID = 9;
export const RANGED_TAG_ID = 10;
export const TOP_OPERATOR_TAG_ID = 11;
export const SENIOR_OPERATOR_TAG_ID = 14;
export const STARTER_TAG_ID = 17;
export const ROBOT_TAG_ID = 28;

export const HIDDEN_TAG_IDS = new Set<number>([1012, 1013]);
export const HIDDEN_TAG_NAMES = new Set<string>(["Male", "Female"]);

/**
 * Profession labels are game vocabulary (the game data ships its own
 * translation per region), so they are not in the message catalog.
 */
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
