import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

/**
 * The sort names live in a table keyed by sort key, because the select's
 * trigger renders the chosen option through that table rather than from the
 * item it was picked from.
 */
export const namespace = "user";

export const messages = {
    "profile.roster.aria": {
        text: "Operator roster",
        description: "Accessible name of the roster section of a profile.",
    },
    "profile.roster.search.placeholder": {
        text: "Search operators...",
        description: "Prompt inside the empty roster search box. Keep the three dots as written.",
    },
    "profile.roster.sort.placeholder": {
        text: "Sort by",
        description: "Shown in the sort select before anything is chosen.",
    },
    "profile.roster.sort.investment": {
        text: "Sort by Investment",
        description: "Sort option: how far each operator is along its own ceiling (promotion, level, skill, masteries, modules, potential), so a finished lower-rarity operator outranks an untouched higher-rarity one.",
    },
    "profile.roster.sort.level": {
        text: "Sort by Level",
        description: "Sort option: by operator level.",
    },
    "profile.roster.sort.rarity": {
        text: "Sort by Rarity",
        description: "Sort option: by star rating.",
    },
    "profile.roster.sort.obtained": {
        text: "Sort by Obtained",
        description: "Sort option: by when the operator was obtained.",
    },
    "profile.roster.sort.potential": {
        text: "Sort by Potential",
        description: "Sort option: by potential rank. 'Potential' is the game's duplicate-operator system.",
    },
    "profile.roster.sort.trust": {
        text: "Sort by Trust",
        description: "Sort option: by trust percentage. 'Trust' is the game's own affinity stat.",
    },
    "profile.roster.sort.maxed": {
        text: "Sort by Maxed",
        description: "Sort option: fully invested operators first.",
    },
    "profile.roster.sort.asc": {
        text: "Asc",
        description: "Label on the sort-direction button while the order is smallest-first. Abbreviated from 'ascending'; little room.",
    },
    "profile.roster.sort.desc": {
        text: "Desc",
        description: "Label on the sort-direction button while the order is largest-first. Abbreviated from 'descending'; little room.",
    },
    "profile.roster.viewMode.aria": {
        text: "View mode",
        description: "Accessible name of the pair of buttons that switch between card layouts.",
    },
    "profile.roster.viewMode.detailed": {
        text: "Detailed view",
        description: "Accessible name of the button for the large operator cards.",
    },
    "profile.roster.viewMode.compact": {
        text: "Compact view",
        description: "Accessible name of the button for the small operator cards.",
    },
    "profile.roster.showing": {
        text: "Showing {shown} of {total} operators. Scroll to load more.",
        description: "Footer under the roster grid while more cards are still to be loaded.",
    },
} satisfies MessageMap;

// `dynamic`: the sort names are also resolved through the SORT_LABELS table as
// `t(SORT_LABELS[key])`, which the extractor cannot see.
export const { keys } = defineMessages({ namespace, messages, dynamic: true });
