import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "settings";

export const messages = {
    "shell.title": {
        text: "Settings",
        description: "Page title of the settings screen.",
    },
    "shell.subtitle": {
        text: "Manage your account, profile visibility, and app appearance. Changes save automatically.",
        description: "Standfirst under the settings page title.",
    },
    "shell.sectionsNav": {
        text: "Settings sections",
        description: "Accessible name of the sidebar (on mobile, the horizontal strip) that switches settings sections.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
