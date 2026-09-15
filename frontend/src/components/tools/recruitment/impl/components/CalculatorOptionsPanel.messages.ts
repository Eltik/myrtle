import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tools";

export const messages = {
    "recruit.options.includeRobots": {
        text: "Include robots",
        description: "Switch label: whether one-star Robot operators count towards a combination. 'Robot' is the game's own tier.",
    },
    "recruit.options.includeTwoStars": {
        text: "Include 2★ operators",
        description: "Switch label: whether two-star operators count towards a combination.",
    },
    "recruit.options.includeThreeStars": {
        text: "Include 3★ operators",
        description: "Switch label: whether three-star operators count towards a combination.",
    },
    "recruit.options.sort": {
        text: "Sort operators",
        description: "Label over the select that orders the operators listed inside each combination.",
    },
    "recruit.options.sort.placeholder": {
        text: "Sort",
        description: "Placeholder in the sort select before an order is chosen. Very narrow control.",
    },
    "recruit.options.sort.rarityDesc": {
        text: "Highest rarity first",
        description: "Sort option: order operators from the highest star rating down.",
    },
    "recruit.options.sort.commonFirst": {
        text: "Most common first",
        description: "Sort option: order operators from the most frequently rolled up.",
    },
} satisfies MessageMap;

// `dynamic`: the sort-order keys are stored on a constants entry and resolved as
// `t(mode.labelKey)`, so the
// extractor has no literal call site to match them against.
export const { keys } = defineMessages({ namespace, messages, dynamic: true });
