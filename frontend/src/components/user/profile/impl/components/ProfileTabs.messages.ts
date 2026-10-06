import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "user";

export const messages = {
    "profile.tabs.label": {
        text: "Profile sections",
        description: "Accessible name of the profile's tab bar.",
    },
    "profile.tabs.private": {
        text: "Hidden from visitors",
        description: "Accessible label of the eye-off icon the profile owner sees on a tab they made private.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
