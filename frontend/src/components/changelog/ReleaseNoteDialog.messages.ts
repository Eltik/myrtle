import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "changelog";

export const messages = {
    "dialog.alsoInUpdate": {
        text: "Also in this update",
        description: "Heading above the secondary bullet list in the what's-new dialog. Rendered uppercase.",
    },
    "dialog.allUpdates": {
        text: "All updates",
        description: "Link at the foot of the what's-new dialog; opens the full changelog.",
    },
    "dialog.allUpdates.missed": {
        text: "All updates ({count} more you missed)",
        description: "Same link as 'All updates', shown when earlier announcements went unread. {count} is rendered unformatted and the phrase is always plural in the source.",
    },
    "dialog.gotIt": {
        text: "Got it",
        description: "Button that dismisses the what's-new dialog without following its link.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
