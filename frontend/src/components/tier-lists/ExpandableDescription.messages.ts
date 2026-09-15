import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tierLists";

export const messages = {
    "description.showMore": {
        text: "Show more",
        description: "Button that expands a clamped description to its full height. Rendered uppercase.",
    },
    "description.showLess": {
        text: "Show less",
        description: "Button that collapses an expanded description back to a few lines. Rendered uppercase.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
