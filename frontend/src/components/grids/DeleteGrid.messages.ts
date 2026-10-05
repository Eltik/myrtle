import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "grids";

// The keys keep their original `my.` prefix: they moved here from MyGrids, and renaming them would orphan every existing translation.
export const messages = {
    "delete.action": {
        text: "Delete",
        description: "Labelled button on a grid's page and in its editor that deletes the grid (after a confirmation). May show as an icon only on narrow screens.",
    },
    "my.delete": {
        text: "Delete {title}",
        description: "Accessible name and tooltip of a card's delete icon button. {title} is the grid's title.",
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
