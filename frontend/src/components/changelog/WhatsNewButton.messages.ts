import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "changelog";

export const messages = {
    "whatsNew.aria": {
        text: "What's new",
        description: "Accessible name of the bell button in the site header, which opens the latest release note.",
    },
    "whatsNew.aria.unread": {
        text: "What's new (unread)",
        description: "Accessible name of the header bell button while an announcement has not been opened yet. The bracketed word tells a screen-reader user what the dot means.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
