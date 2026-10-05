import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "grids";

export const messages = {
    "my.kicker": {
        text: "Your grids",
        description: "Small uppercase label above the heading of the signed-in user's grids.",
    },
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
    "my.delete": {
        text: "Delete {title}",
        description: "Accessible name and tooltip of a card's delete icon button. {title} is the grid's title.",
    },
    "my.error": {
        text: "Your grids could not be loaded.",
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
    "my.deleteDialog.title": {
        text: "Delete this grid?",
        description: "Title of the delete confirmation.",
    },
    "my.deleteDialog.body": {
        text: '"{title}" will be deleted for good. Grids made from it keep their own copy.',
        description: "Body of the delete confirmation. {title} is the grid's title; keep the quotes.",
    },
    "my.deleteDialog.confirm": {
        text: "Delete",
        description: "Confirms deleting the grid.",
    },
    "my.deleteDialog.cancel": {
        text: "Cancel",
        description: "Closes the delete confirmation.",
    },
    "my.toast.deleted": {
        text: 'Deleted "{title}"',
        description: "Toast after a grid was deleted. {title} is its title; keep the quotes.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
