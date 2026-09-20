import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "user";

export const messages = {
    "leaderboard.mostHeld.title": {
        text: "Most held",
        description: "Heading of the sidebar card listing the items held by the most players.",
    },
    "leaderboard.mostHeld.subtitle": {
        text: "Items by how many players hold them",
        description: "Subtitle under the 'Most held' heading.",
    },
    "leaderboard.mostHeld.holders": {
        text: "{count, number} {count, plural, one {holder} other {holders}}",
        description: "Holder count on each row of the 'Most held' card. {count} is the number of players holding the item.",
    },
    "leaderboard.mostHeld.top": {
        text: "top {top}",
        description: "The largest single holding on each row of the 'Most held' card. {top} is a compact number like 230k.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
