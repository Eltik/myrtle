import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "skins";

export const messages = {
    "detail.srTitle": {
        text: "{op} - {skin}",
        description: "Screen-reader-only title of the skin detail dialog. Both values are names from the game data and are never translated.",
    },
    "detail.heroAlt": {
        text: "{op} {skin}",
        description: "Alt text of the full-size skin artwork. Both values are names from the game data and are never translated.",
    },
    "detail.back": {
        text: "Back",
        description: "Default label of the mobile-only close button at the foot of the skin detail dialog; it returns to the list the dialog was opened from.",
    },
    "detail.row.price": {
        text: "Price",
        description: "Row label in the skin detail dialog: what the outfit costs. Rendered uppercase in a narrow column.",
    },
    "detail.row.obtain": {
        text: "Obtain",
        description: "Row label in the skin detail dialog: how the outfit is acquired. A noun heading, not a verb. Rendered uppercase.",
    },
    "detail.row.usage": {
        text: "Usage",
        description: "Row label in the skin detail dialog, above the outfit's in-game usage note from the game data. Rendered uppercase.",
    },
    "detail.row.description": {
        text: "Description",
        description: "Row label in the skin detail dialog, above the outfit's description from the game data. Rendered uppercase.",
    },
    "detail.row.dialog": {
        text: "Dialog",
        description: "Row label in the skin detail dialog, above the quoted in-game line that comes with the outfit. Rendered uppercase.",
    },
    "detail.row.credits": {
        text: "Credits",
        description: "Row label in the skin detail dialog, above the artists who made the outfit. Rendered uppercase.",
    },
    "detail.row.released": {
        text: "Released",
        description: "Row label in the skin detail dialog, above the outfit's release date. Rendered uppercase.",
    },
    "detail.credits.art": {
        text: "Art: {names}",
        description: "Illustrator credit in the skin detail dialog. {names} is a comma-joined list of artist names, which are never translated.",
    },
    "detail.credits.design": {
        text: "Design: {names}",
        description: "Designer credit in the skin detail dialog. {names} is a comma-joined list of designer names, which are never translated.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
