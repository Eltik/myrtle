import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tierLists";

export const messages = {
    "my.row.emptyPreview": {
        text: "empty",
        description: "Stand-in where a row's operator thumbnails would be, when the list holds none. Lowercase and italic by design.",
    },
    "my.row.untitled": {
        text: "Untitled list",
        description: "Stand-in title for a list whose title is still empty.",
    },
    "my.row.official": {
        text: "Official",
        description: "Badge marking a list maintained for the site's own team. Rendered uppercase.",
    },
    "my.row.draft": {
        text: "Draft",
        description: "Badge on a row whose list has no operators placed yet. Rendered uppercase.",
    },
    "my.row.actions": {
        text: "List actions",
        description: "Accessible name of the button that opens the row's action menu.",
    },
    "my.row.open": {
        text: "Open",
        description: "Menu item that opens the list's public page.",
    },
    "my.row.editBoard": {
        text: "Edit Tierlist",
        description: "Menu item that opens the tier board editor, where operators are placed.",
    },
    "my.row.editDetails": {
        text: "Edit details",
        description: "Menu item that opens the dialog for renaming the list and editing its description.",
    },
    "my.row.copyLink": {
        text: "Copy share link",
        description: "Menu item that copies the list's public link to the clipboard.",
    },
    "my.row.delete": {
        text: "Delete",
        description: "Menu item that starts deleting the list.",
    },
    "my.row.views": {
        text: "{count} views",
        description: "Tooltip on a row's view count.",
    },
    "my.row.favorites": {
        text: "{count} favorites",
        description: "Tooltip on a row's favourite count.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
