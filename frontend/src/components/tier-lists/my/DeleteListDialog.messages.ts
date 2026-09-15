import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tierLists";

export const messages = {
    "my.delete.title": {
        text: "Delete tier list?",
        description: "Title of the dialog confirming deletion of a tier list.",
    },
    "my.delete.fallbackName": {
        text: "This list",
        description: "Stand-in for the list's name in the confirmation sentence when the name is not to hand.",
    },
    "my.delete.body": {
        text: "{name} and all of its tiers, placements, and stats will be permanently removed. Anyone with the link will see a 404 page. This cannot be undone.",
        description: "Confirmation sentence in the delete dialog. {name} is the list's own name in bold, or my.delete.fallbackName when it is not to hand, and may move wherever the sentence needs it. '404' is the web's not-found status and stays as-is.",
    },
    "my.delete.cancel": {
        text: "Cancel",
        description: "Button that closes the delete dialog without deleting anything.",
    },
    "my.delete.submit": {
        text: "Delete list",
        description: "Button that deletes the tier list for good.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
