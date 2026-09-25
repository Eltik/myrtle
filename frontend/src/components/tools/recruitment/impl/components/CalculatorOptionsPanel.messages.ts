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
    "recruit.options.sort.potentialAsc": {
        text: "Lowest potential first",
        description: "Sort option, offered only when signed in: operators the roster does not hold first, then by the potential the roster holds them at, lowest first. Maxed operators last.",
    },
    "recruit.options.layout": {
        text: "Layout",
        description: "Label over the select that picks how the results are drawn. Saved per browser.",
    },
    "recruit.options.layout.compact": {
        text: "Compact",
        description: "Layout option: one row per tag combination, operators as small portrait tiles.",
    },
    "recruit.options.layout.detailed": {
        text: "Detailed",
        description: "Layout option: one card per tag combination, each operator a wide row with stars and class.",
    },
    "recruit.options.showPotentials": {
        text: "Show my potentials",
        description: "Switch label: overlay the signed-in user's current potential (P1 to P6) on each operator in the results.",
    },
    "recruit.options.showNextUpgrade": {
        text: "Show next potential",
        description: "Switch label: under each operator's portrait, show what their next potential rank would grant (e.g. 'DP cost -1', 'Talent 2'). Saved per browser.",
    },
    "recruit.options.roster.signIn": {
        text: "Sign in to sync your roster",
        description: "Hint under the roster switches when nobody is signed in; the switches are disabled.",
    },
} satisfies MessageMap;

// `dynamic`: the sort-order and layout keys are stored on a constants entry and resolved as
// `t(mode.labelKey)`, so the
// extractor has no literal call site to match them against.
export const { keys } = defineMessages({ namespace, messages, dynamic: true });
