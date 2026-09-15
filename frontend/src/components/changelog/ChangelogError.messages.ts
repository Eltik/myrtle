import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "changelog";

export const messages = {
    "error.unexpected": {
        text: "An unexpected error occurred.",
        description: "Stand-in shown in the changelog error screen's detail box when the failure carried no message of its own.",
    },
    "error.title": {
        text: "Couldn't load the changelog",
        description: "Heading of the changelog error screen. No full stop in the source.",
    },
    "error.body": {
        text: "We couldn't reach GitHub to fetch the latest commits. This is usually a temporary network hiccup or an API rate limit - try again in a moment.",
        description: "Explanation under the changelog error heading. 'GitHub' is a product name and stays as-is.",
    },
    "error.retry": {
        text: "Try again",
        description: "Primary button on the changelog error screen; refetches the commits.",
    },
    "error.home": {
        text: "Return home",
        description: "Secondary button on the changelog error screen; goes to the landing page.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
