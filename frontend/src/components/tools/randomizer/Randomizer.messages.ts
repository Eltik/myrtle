import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tools";

export const messages = {
    "randomizer.breadcrumb.tools": {
        text: "Tools",
        description: "First crumb of the breadcrumb trail, naming the section this tool lives in.",
    },
    "randomizer.breadcrumb.title": {
        text: "Randomizer",
        description: "Last breadcrumb, naming this tool. The page heading itself is a separate string.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
