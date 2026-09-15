import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

/**
 * The option lists in `constants.ts` are plain data in a module with no React,
 * so they carry message KEYS and whichever component renders a row resolves it
 * with `t()`.
 *
 * Only this site's own chrome lives here. The class, archetype, nation and
 * faction option labels are game vocabulary - they come from
 * `formatProfession` and friends in `lib/utils.ts`, which the game-data layer
 * translates per region - so they are deliberately absent.
 */
export const namespace = "operators";

export const messages = {
    "filters.notes.any": {
        text: "Any",
        description: "Notes filter option: do not filter on whether the operator has notes.",
    },
    "filters.notes.yes": {
        text: "Has notes",
        description: "Notes filter option: only operators this site has written notes for.",
    },
    "filters.notes.no": {
        text: "No notes",
        description: "Notes filter option: only operators with no notes.",
    },
    "sort.rarity": {
        text: "Rarity",
        description: "Sort option: by star rating. Short label in a compact select.",
    },
    "sort.name": {
        text: "Name",
        description: "Sort option: alphabetically by operator name.",
    },
    "sort.class": {
        text: "Class",
        description: "Sort option: by the operator's class (Guard, Sniper, ...). Short label in a compact select.",
    },
    "sort.hp": {
        text: "HP",
        description: "Sort option: by max health. Abbreviation of 'hit points'; the game uses HP in every region, so it normally stays as-is.",
    },
    "sort.atk": {
        text: "ATK",
        description: "Sort option: by attack power. The game's own abbreviation, which normally stays as-is.",
    },
    "sort.def": {
        text: "DEF",
        description: "Sort option: by defense. The game's own abbreviation, which normally stays as-is.",
    },
    "sort.res": {
        text: "RES",
        description: "Sort option: by arts resistance. The game's own abbreviation, which normally stays as-is.",
    },
    "sort.cost": {
        text: "Cost",
        description: "Sort option: by deployment cost.",
    },
    "sort.block": {
        text: "Block",
        description: "Sort option: by how many enemies the operator blocks.",
    },
    "sort.ownership": {
        text: "Most owned",
        description: "Sort option: by the share of players who own the operator.",
    },
    "sort.e2": {
        text: "Most E2''d",
        description: "Sort option: by the share of owners who promoted the operator to Elite 2. 'E2' is the game's name for the second promotion; the doubled apostrophe is ICU escaping and renders as one.",
    },
} satisfies MessageMap;

// `dynamic`: these keys are stored on a registry/constants entry and resolved
// by the consuming component as `t(item.labelKey)`, so the extractor has no
// literal call site to match them against.
export const { keys } = defineMessages({ namespace, messages, dynamic: true });
