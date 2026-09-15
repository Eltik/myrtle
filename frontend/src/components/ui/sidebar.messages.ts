import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

/** `ui/` primitives share one namespace: their strings are the site's chrome. */
export const namespace = "common";

export const messages = {
    "sidebar.title": {
        text: "Sidebar",
        description: "Screen-reader-only title of the sheet the sidebar becomes on small screens. Never visible.",
    },
    "sidebar.description": {
        text: "Displays the mobile sidebar.",
        description: "Screen-reader-only description of that same sheet. Never visible.",
    },
    "sidebar.toggle": {
        text: "Toggle Sidebar",
        description: "Accessible name and tooltip of the control that collapses or expands the sidebar. A verb: one control does both directions.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
