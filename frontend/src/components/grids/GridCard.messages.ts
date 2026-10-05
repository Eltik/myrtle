import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "grids";

export const messages = {
    "card.by": {
        text: "by {name}",
        description: "Byline on a grid card. {name} is the owner's display name.",
    },
    "card.forks": {
        text: "{count, plural, one {Used as a template {formatted} time} other {Used as a template {formatted} times}}",
        description: "Tooltip of the fork count on a grid card. {formatted} is {count} formatted for the locale.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
