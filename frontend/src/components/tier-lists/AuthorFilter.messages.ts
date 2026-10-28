import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tierLists";

export const messages = {
    "browse.author.label": {
        text: "Filter by author",
        description: "Accessible name of the box on the tier list index that shows only one player's lists.",
    },
    "browse.author.placeholder": {
        text: "Any author",
        description: "Placeholder in the author filter when no one is picked. Keep it short: the box is narrow.",
    },
    "browse.author.empty": {
        text: "No player matches",
        description: "Shown in the author filter's menu when the search finds no one.",
    },
    "browse.author.searching": {
        text: "Searching…",
        description: "Shown in the author filter's menu while players are being looked up.",
    },
    "browse.author.lists": {
        text: "{count, plural, one {# list} other {# lists}}",
        description: "Beside a player in the author filter: how many public tier lists they have.",
    },
    "browse.author.unknown": {
        text: "Selected player",
        description: "Stands in for the author's name in the filter when a shared link names a player with no public lists.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
