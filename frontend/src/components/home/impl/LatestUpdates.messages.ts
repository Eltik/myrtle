import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "home";

export const messages = {
    "updates.eyebrow": {
        text: "What’s new",
        description: "Small uppercase label above the home page's latest release notes panel.",
    },
    "updates.title": {
        text: "Latest updates",
        description: "Heading of the home page panel listing the newest release notes.",
    },
    "updates.seeAll": {
        text: "Changelog →",
        description: "Link from the home page's latest-updates panel to the full changelog page. The arrow is part of the label.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
