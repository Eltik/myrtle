import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tierLists";

export const messages = {
    "edit.publishDialog.title": {
        text: "Publish a version",
        description: "Title of the dialog that snapshots the list as a new version.",
    },
    "edit.publishDialog.description": {
        text: "Snapshots the current tiers and placements as version v{version}. Older versions stay accessible to viewers.",
        description: "Explains what publishing does. Keep the 'v' before the version number.",
    },
    "edit.publishDialog.latest": {
        text: "Latest published:",
        description: "Label before the most recently published version number. Keep the colon.",
    },
    "edit.publishDialog.next": {
        text: "Next:",
        description: "Label before the version number this publish will create. Keep the colon.",
    },
    "edit.publishDialog.changelog": {
        text: "Changelog",
        description: "Label of the field describing what changed in this version.",
    },
    "edit.publishDialog.changelogPlaceholder": {
        text: "What changed in this version? e.g. Promoted Texas to S, added Wis'adel.",
        description: "Placeholder in the changelog field. Texas and Wis'adel are operator names from the game and stay as they are.",
    },
    "edit.publishDialog.changelogHint": {
        text: "Optional but recommended. Viewers see this on the version history.",
        description: "Explains the changelog field, under it.",
    },
    "edit.publishDialog.cancel": {
        text: "Cancel",
        description: "Button that closes the publish dialog without publishing.",
    },
    "edit.publishDialog.submit": {
        text: "Publish v{version}",
        description: "Button that publishes the snapshot, naming the version it will create. Keep the 'v' before the number.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
