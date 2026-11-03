import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "grids";

export const messages = {
    "my.title": {
        text: "My Grids",
        description: "Heading of the signed-in user's grids.",
    },
    "my.count": {
        text: "{count, plural, one {# grid} other {# grids}} of {max}",
        description: "How many grids the user has, out of the per-user limit {max} (50).",
    },
    "my.browse": {
        text: "Browse grids",
        description: "Button to the public grid browser.",
    },
    "my.unlisted": {
        text: "Unlisted",
        description: "Badge on a card of a grid hidden from the browse list. Rendered uppercase; keep it short.",
    },
    "my.edit": {
        text: "Edit",
        description: "Opens the grid in the editor.",
    },
    "my.error": {
        text: "Couldn't load your grids.",
        description: "Shown when the user's grids failed to load.",
    },
    "my.retry": {
        text: "Try again",
        description: "Reloads the user's grids after an error.",
    },
    "my.emptyTitle": {
        text: "You have no grids yet.",
        description: "Heading of the empty state of the user's grids.",
    },
    "my.emptyBody": {
        text: "Make one from scratch, or open any grid and use it as a template.",
        description: "Body of the empty state of the user's grids.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
