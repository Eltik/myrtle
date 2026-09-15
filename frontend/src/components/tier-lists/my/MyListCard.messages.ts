import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tierLists";

export const messages = {
    "my.card.emptyDraft": {
        text: "Empty draft",
        description: "Corner badge on a card whose list has no operators placed yet.",
    },
    "my.card.official": {
        text: "Official",
        description: "Corner badge marking a list maintained for the site's own team. Rendered uppercase.",
    },
    "my.card.emptyThumb": {
        text: "No operators placed yet",
        description: "Stand-in inside the card thumbnail when the list holds no operators.",
    },
    "my.card.tier": {
        text: "Tier {name}",
        description: "Tooltip on a tier's pill in the card thumbnail. {name} is the tier's own label, written by the author.",
    },
    "my.card.untitled": {
        text: "Untitled list",
        description: "Stand-in title for a list whose title is still empty.",
    },
    "my.card.actions": {
        text: "List actions",
        description: "Accessible name of the button that opens the card's action menu.",
    },
    "my.card.open": {
        text: "Open",
        description: "Menu item that opens the list's public page.",
    },
    "my.card.editBoard": {
        text: "Edit Tierlist",
        description: "Menu item that opens the tier board editor, where operators are placed.",
    },
    "my.card.editDetails": {
        text: "Edit details",
        description: "Menu item that opens the dialog for renaming the list and editing its description.",
    },
    "my.card.copyLink": {
        text: "Copy share link",
        description: "Menu item that copies the list's public link to the clipboard.",
    },
    "my.card.delete": {
        text: "Delete",
        description: "Menu item that starts deleting the list.",
    },
    "my.card.views": {
        text: "{count} views",
        description: "Tooltip on a card's view count.",
    },
    "my.card.favorites": {
        text: "{count} favorites",
        description: "Tooltip on a card's favourite count.",
    },
    "my.card.views24h": {
        text: "{count} views in the last 24h",
        description: "Tooltip on a card's recent view count.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
