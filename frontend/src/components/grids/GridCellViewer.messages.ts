import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "grids";

export const messages = {
    "viewer.position": {
        text: "Row {row}, column {col}",
        description: "Top of the full-screen cell viewer: where the shown cell sits on the grid. {row} and {col} are 1-based.",
    },
    "viewer.count": {
        text: "{n} of {total}",
        description: "Top of the full-screen cell viewer: which of the grid's filled cells is shown. {n} is its place among them, {total} how many there are.",
    },
    "viewer.hint": {
        text: "Use the arrow keys or swipe to move between cells.",
        description: "Screen-reader description of the full-screen cell viewer, read when it opens.",
    },
    "viewer.close": {
        text: "Close",
        description: "Accessible name and tooltip of the X button that closes the full-screen cell viewer.",
    },
    "viewer.prev": {
        text: "Previous cell",
        description: "Accessible name and tooltip of the button that shows the grid's previous filled cell in the full-screen viewer.",
    },
    "viewer.next": {
        text: "Next cell",
        description: "Accessible name and tooltip of the button that shows the grid's next filled cell in the full-screen viewer.",
    },
    "viewer.untitledCell": {
        text: "Cell at row {row}, column {col}",
        description: "Title of the full-screen cell viewer for a cell that has a pick but no label. {row} and {col} are 1-based.",
    },
    "viewer.noPick": {
        text: "Nothing picked for this cell",
        description: "Shown in the full-screen cell viewer under a cell's label when the cell holds no pick.",
    },
    "viewer.unknown": {
        text: "Not in this server's game data",
        description: "Shown in the full-screen cell viewer for a pick the reader's game data does not know; the raw id is shown beside it.",
    },
    "viewer.openPage": {
        text: "Open page",
        description: "Link in the full-screen cell viewer to the picked thing's own page on the site (an operator's page, an enemy's handbook entry).",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
