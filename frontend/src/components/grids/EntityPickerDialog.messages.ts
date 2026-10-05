import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "grids";

export const messages = {
    "picker.title": {
        text: "Pick for row {row}, column {col}",
        description: "Title of the grid editor's picker dialog when the cell has no label. {row} and {col} are 1-based positions.",
    },
    "picker.titleLabelled": {
        text: "Pick: {label}",
        description: "Title of the grid editor's picker dialog. {label} is the cell's own label, e.g. 'Favorite Vanguard'.",
    },
    "picker.description": {
        text: "Choose an operator, skin, event or anything else for this cell.",
        description: "Line under the picker dialog's title.",
    },
    "picker.kinds": {
        text: "What to pick from",
        description: "Accessible name of the row of kind tabs (Operators, Skins, Events...) in the picker dialog.",
    },
    "picker.searchPlaceholder": {
        text: "Search {kind}",
        description: "Placeholder of the picker's search box. {kind} is the current tab's plural name, e.g. 'Operators'.",
    },
    "picker.loading": {
        text: "Loading…",
        description: "Shown in the picker while the current tab's list loads.",
    },
    "picker.error": {
        text: "This list could not be loaded.",
        description: "Shown in the picker when the current tab's list failed to load.",
    },
    "picker.retry": {
        text: "Retry",
        description: "Button that reloads the picker's failed list.",
    },
    "picker.empty": {
        text: "Nothing matches your search.",
        description: "Shown in the picker when the search leaves nothing.",
    },
    "picker.showMore": {
        text: "Show more ({shown} of {total})",
        description: "Button under a long picker list that renders the next batch. {shown} is how many are shown, {total} how many match.",
    },
    "picker.clear": {
        text: "Clear",
        description: "Button in the picker that removes the cell's current pick, keeping its label.",
    },
    "picker.cancel": {
        text: "Cancel",
        description: "Button that closes the picker without changing the cell.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
