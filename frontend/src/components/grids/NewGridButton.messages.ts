import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "grids";

export const messages = {
    "create.open": {
        text: "New grid",
        description: "Button that opens the create-grid dialog (or the sign-in dialog when signed out).",
    },
    "create.title": {
        text: "New grid",
        description: "Title of the create-grid dialog.",
    },
    "create.description": {
        text: "Start blank or from a ready-made set of labels. You can change everything later.",
        description: "Line under the create-grid dialog's title.",
    },
    "create.starter": {
        text: "Start from",
        description: "Label over the two starter choices in the create-grid dialog.",
    },
    "create.starter.blank": {
        text: "Blank",
        description: "Starter choice: an empty grid of the chosen size.",
    },
    "create.starter.blankHint": {
        text: "Empty cells, your own labels.",
        description: "Hint under the Blank starter choice.",
    },
    "create.starter.aboutMe": {
        text: "About Me (Arknights)",
        description: "Starter choice: a grid pre-filled with 36 'favorite' prompts about Arknights. 'Arknights' is the game's name and stays as-is.",
    },
    "create.starter.aboutMeHint": {
        text: "A {size} by {size} of favorites to fill in.",
        description: "Hint under the About Me starter. {size} is 6: the starter is always 6 by 6.",
    },
    "create.aboutMeTitle": {
        text: "About Me",
        description: "Title filled into the empty title field when the About Me starter is chosen.",
    },
    "create.name": {
        text: "Title",
        description: "Label of the title field in the create-grid dialog.",
    },
    "create.namePlaceholder": {
        text: "e.g. My Arknights favorites",
        description: "Placeholder of the title field in the create-grid dialog.",
    },
    "create.rows": {
        text: "Rows",
        description: "Label of the rows stepper in the create-grid dialog.",
    },
    "create.cols": {
        text: "Columns",
        description: "Label of the columns stepper in the create-grid dialog.",
    },
    "create.cancel": {
        text: "Cancel",
        description: "Closes the create-grid dialog.",
    },
    "create.submit": {
        text: "Create and edit",
        description: "Creates the grid and opens it in the editor.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
