import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "user";

/** Names of the metric sorts in `searchControls.ts`, one per `METRIC_SORTS` entry. */
export const messages = {
    "search.sort.score": {
        text: "Score",
        description: "Rank-by option and status-line label: the site's total profile score, the default order.",
    },
    "search.sort.operators": {
        text: "Operators owned",
        description: "Rank-by option: how many operators the player owns.",
    },
    "search.sort.joined": {
        text: "Joined the game",
        description: "Rank-by option: the date the game account was registered.",
    },
    "search.sort.enemies": {
        text: "Enemies discovered",
        description: "Rank-by option: how many enemy entries the player has unlocked.",
    },
    "search.sort.potentials": {
        text: "Potentials",
        description: "Rank-by option: potential ranks summed over the roster.",
    },
    "search.sort.masteries": {
        text: "Masteries (M3)",
        description: "Rank-by option: skills at mastery 3. Keep 'M3', it is the game's shorthand.",
    },
    "search.sort.modules": {
        text: "Modules (Lv3)",
        description: "Rank-by option: modules at level 3. Keep 'Lv3'.",
    },
    "search.sort.skins": {
        text: "Skins owned",
        description: "Rank-by option: how many skins the player owns.",
    },
} satisfies MessageMap;

// `dynamic`: these keys are stored on `METRIC_SORT_LABEL_KEYS` and resolved
// by the toolbar as `t(METRIC_SORT_LABEL_KEYS[sort])`, so the extractor has no
// literal call site to match them against.
export const { keys } = defineMessages({ namespace, messages, dynamic: true });
