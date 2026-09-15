import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "admin";

export const messages = {
    "topbar.openMenu": {
        text: "Open menu",
        description: "Accessible name of the hamburger button that opens the admin sidebar on a narrow screen.",
    },
    "topbar.breadcrumb": {
        text: "Breadcrumb",
        description: "Accessible name of the breadcrumb navigation landmark in the admin top bar.",
    },
    "topbar.openSearch": {
        text: "Open search",
        description: "Accessible name of the button that opens the command palette from the admin top bar.",
    },
    "topbar.searchPlaceholder": {
        text: "Search users, tier lists, operators…",
        description: "Prompt inside the admin search button. 'Operators' are the game's playable characters. Keep the ellipsis character.",
    },
    "topbar.notifications": {
        text: "Notifications",
        description: "Accessible name of the bell button in the admin top bar.",
    },
    "topbar.github": {
        text: "GitHub",
        description: "Accessible name of the link to the project's source repository. 'GitHub' is a product name and stays as-is.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
