import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tierLists";

export const messages = {
    "my.create.title": {
        text: "New tier list",
        description: "Title of the dialog for creating a tier list.",
    },
    "my.create.description": {
        text: "Give your list a name and a short description. You can change these later, and your edits will be visible to everyone who has the share link.",
        description: "Explains the create dialog.",
    },
    "my.create.name": {
        text: "Name",
        description: "Field label for the new list's title.",
    },
    "my.create.namePlaceholder": {
        text: "e.g. Endgame DPS rankings",
        description: "Placeholder in the name field, giving an example title. 'DPS' is the community's abbreviation for damage per second.",
    },
    "my.create.nameHint": {
        text: "Shown on browse cards and on the public detail page.",
        description: "Explains where the list's name appears.",
    },
    "my.create.descriptionLabel": {
        text: "Description",
        description: "Field label for the new list's description.",
    },
    "my.create.descriptionPlaceholder": {
        text: "What's this list about? Who is it for?",
        description: "Placeholder in the description field.",
    },
    "my.create.descriptionHint": {
        text: "Optional. A sentence or two helps readers know what to expect.",
        description: "Explains the description field.",
    },
    "my.create.cancel": {
        text: "Cancel",
        description: "Button that closes the create dialog without creating anything.",
    },
    "my.create.submit": {
        text: "Create list",
        description: "Button that creates the tier list.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
