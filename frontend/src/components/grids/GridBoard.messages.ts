import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "grids";

export const messages = {
    "board.label": {
        text: "{title}, a grid of {rows} by {cols}",
        description: "Accessible name of a grid board. {title} is the grid's own title; {rows} and {cols} are its row and column counts.",
    },
    "board.untitled": {
        text: "Untitled grid",
        description: "Shown as a grid's title on the board while its title is empty.",
    },
    "cell.addItem": {
        text: "Add Item",
        description: "Centred in an empty cell's grey square in the grid editor; clicking it opens the picker. Keep it short: cells can be under 50 pixels wide.",
    },
    "cell.pick": {
        text: "Pick something for row {row}, column {col}",
        description: "Accessible name of an empty cell's art area in the grid editor.",
    },
    "cell.change": {
        text: "{name} in row {row}, column {col}. Change it",
        description: "Accessible name of a filled cell's art area in the grid editor. {name} is the picked entity's name and kind from the game data.",
    },
    "cell.clear": {
        text: "Remove {name} from row {row}, column {col}",
        description: "Accessible name and tooltip of the small x button on a filled cell in the grid editor. It removes the pick at once and keeps the cell's label. {name} is the picked entity's name.",
    },
    "cell.editLabel": {
        text: "Label of row {row}, column {col}: {label}. Edit it",
        description: "Accessible name of a cell's label strip in the grid editor, which turns into a text field when clicked. {label} is the current label.",
    },
    "cell.noLabel": {
        text: "none",
        description: "Stands in for {label} in the label strip's accessible name when the cell has no label yet.",
    },
    "cell.labelPlaceholder": {
        text: "Add label",
        description: "Placeholder in an empty label strip in the grid editor, and in the label text field. Keep it short: cells can be under 50 pixels wide.",
    },
    "cell.view.both": {
        text: "{label}: {name}. Row {row}, column {col}. Open it full screen",
        description: "Accessible name of a cell on a read-only grid that has a label and a pick; activating it opens the full-screen cell viewer. {label} is the cell's label, {name} the picked entity's name and kind.",
    },
    "cell.view.label": {
        text: "{label}. Row {row}, column {col}. Open it full screen",
        description: "Accessible name of a cell on a read-only grid that has a label but no pick; activating it opens the full-screen cell viewer.",
    },
    "cell.view.pick": {
        text: "{name}. Row {row}, column {col}. Open it full screen",
        description: "Accessible name of a cell on a read-only grid that has a pick but no label; activating it opens the full-screen cell viewer. {name} is the picked entity's name and kind.",
    },
    "cell.open": {
        text: "Edit row {row}, column {col}",
        description: "Accessible name of a cell's art and label in the grid editor on a phone, where tapping either opens the full-screen cell editor.",
    },
    "cell.labelInput": {
        text: "Label of row {row}, column {col}",
        description: "Accessible name of the label text field that replaces a cell's strip while it is being edited.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
