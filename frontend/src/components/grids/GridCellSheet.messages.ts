import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "grids";

export const messages = {
    "sheet.title": {
        text: "Row {row}, column {col}",
        description: "Title of the full-screen cell editor a phone opens when a grid cell is tapped in the editor. {row} and {col} are 1-based.",
    },
    "sheet.count": {
        text: "Cell {n} of {total}",
        description: "In the phone cell editor: which cell of the board is open, counting row by row. {total} is every cell on the board.",
    },
    "sheet.description": {
        text: "Edit this cell's label and pick. Changes are kept until you save the grid.",
        description: "Screen-reader description of the phone cell editor.",
    },
    "sheet.done": {
        text: "Done",
        description: "Closes the phone cell editor and goes back to the board. It does not save the grid; the editor's Save does.",
    },
    "sheet.empty": {
        text: "Nothing picked yet",
        description: "In the phone cell editor's preview when the cell holds no pick.",
    },
    "sheet.pick": {
        text: "Pick",
        description: "Button in the phone cell editor that opens the picker for an empty cell.",
    },
    "sheet.change": {
        text: "Change",
        description: "Button in the phone cell editor that opens the picker to replace the cell's current pick.",
    },
    "sheet.clear": {
        text: "Clear",
        description: "Button in the phone cell editor that removes the cell's pick at once and keeps its label.",
    },
    "sheet.label": {
        text: "Label",
        description: "Label of the text field for the cell's label in the phone cell editor.",
    },
    "sheet.counter": {
        text: "{count}/{max}",
        description: "Under the phone cell editor's label field: characters used of the limit. {count} and {max} are numbers.",
    },
    "sheet.prev": {
        text: "Previous cell",
        description: "Button in the phone cell editor that opens the board's previous cell, counting row by row.",
    },
    "sheet.next": {
        text: "Next cell",
        description: "Button in the phone cell editor that opens the board's next cell, counting row by row.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
