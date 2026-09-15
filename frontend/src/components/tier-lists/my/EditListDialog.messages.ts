import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tierLists";

export const messages = {
    "my.edit.title": {
        text: "Edit list details",
        description: "Title of the dialog for renaming a list and editing its description.",
    },
    "my.edit.description": {
        text: "Rename the list and update its description. Changes go live immediately.",
        description: "Explains the edit-details dialog.",
    },
    "my.edit.name": {
        text: "Name",
        description: "Field label for the list's title.",
    },
    "my.edit.nameHint": {
        text: "The URL slug will not change when you rename the list.",
        description: "Explains that renaming does not change the list's link. 'Slug' is the readable part of the URL.",
    },
    "my.edit.descriptionLabel": {
        text: "Description",
        description: "Field label for the list's description.",
    },
    "my.edit.descriptionPlaceholder": {
        text: "A sentence or two about this list.",
        description: "Placeholder in the description field.",
    },
    "my.edit.cancel": {
        text: "Cancel",
        description: "Button that closes the dialog without saving.",
    },
    "my.edit.submit": {
        text: "Save changes",
        description: "Button that writes the new name and description.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
