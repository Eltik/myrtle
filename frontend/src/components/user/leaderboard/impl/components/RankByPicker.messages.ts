import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "user";

export const messages = {
    "leaderboard.rankBy.label": {
        text: "Rank by",
        description: "Small label on the control that chooses what the leaderboard is ranked by: a score, or how much of one item players hold.",
    },
    "leaderboard.rankBy.aria": {
        text: "Choose what to rank by",
        description: "Accessible name of the 'Rank by' control.",
    },
    "leaderboard.rankBy.search.placeholder": {
        text: "Search scores and items…",
        description: "Placeholder of the search box inside the 'Rank by' picker.",
    },
    "leaderboard.rankBy.search.label": {
        text: "Search rankings",
        description: "Accessible label of the search box inside the 'Rank by' picker.",
    },
    "leaderboard.rankBy.search.clear": {
        text: "Clear search",
        description: "Accessible label of the button that empties the picker's search box.",
    },
    "leaderboard.rankBy.group.score": {
        text: "Score",
        description: "Heading of the picker group listing the score metrics (total, operators, stages, …).",
    },
    "leaderboard.rankBy.group.items": {
        text: "Inventory",
        description: "Heading of the picker group listing items players can be ranked by (Originite Prime, Orundum, …).",
    },
    "leaderboard.rankBy.group.items.hint": {
        text: "Type to search all {count, number} items",
        description: "Hint under the Inventory group heading before the user searches. {count} is how many distinct items at least one player holds.",
    },
    "leaderboard.rankBy.holders": {
        text: "{count, number} {count, plural, one {holder} other {holders}}",
        description: "Holder count shown next to each item in the picker. {count} is the number of players holding the item.",
    },
    "leaderboard.rankBy.held": {
        text: "{total} held",
        description: "Follows the holder count next to each item in the picker, after a separator: how much of the item every ranked player holds together. {total} is a compact number like 544.8k.",
    },
    "leaderboard.rankBy.line.title": {
        text: "{holders, number} of {population, number} players hold {total, number} in all, {share}",
        description: "Hover text and screen-reader reading of an item's line in the picker. {holders} hold the item, {population} is every visible player, {total} is their holdings summed, {share} is the percentage already formatted.",
    },
    "leaderboard.rankBy.empty": {
        text: "Nothing matches.",
        description: "Shown in the picker when the search matches neither a score metric nor an item.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
