import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "user";

export const messages = {
    "profile.tabs.label": {
        text: "Profile sections",
        description: "Accessible name of the profile's tab bar.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
