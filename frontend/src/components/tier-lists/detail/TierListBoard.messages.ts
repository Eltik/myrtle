import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tierLists";

export const messages = {
    "detail.board.emptyTitle": {
        text: "This list has no tiers yet.",
        description: "Shown in place of the board when the list's author has not added any tiers.",
    },
    "detail.board.emptyBody": {
        text: "The author hasn't published any tiers for this list.",
        description: "Second line of the empty board state.",
    },
    "detail.board.label": {
        text: "Tier list {title}",
        description: "Accessible name of the board. {title} is the list's own title, written by its author.",
    },
    "detail.board.tierCount": {
        text: "{count, plural, one {tier} other {tiers}}",
        description: "Follows the tier count under the board; the number itself is rendered just before it.",
    },
    "detail.board.operatorCount": {
        text: "{count, plural, one {operator} other {operators}} placed",
        description: "Follows the operator count under the board; the number itself is rendered just before it.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
