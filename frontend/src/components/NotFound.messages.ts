import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "common";

export const messages = {
    "notFound.title": {
        text: "Lost in the Wastes",
        description: "Heading of the 404 page. A Terra-flavoured 'page not found'; a literal translation is fine if the setting has no equivalent.",
    },
    "notFound.description": {
        text: "The page you are looking for does not exist, has been moved, or is temporarily unavailable.",
        description: "Body copy of the 404 page.",
    },
    "notFound.returnHome": {
        text: "Return Home",
        description: "Primary 404 action: navigates to the site root.",
    },
    "notFound.searchOperators": {
        text: "Search operators",
        description: "Secondary 404 action: opens the command palette.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
