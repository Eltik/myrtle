import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "legal";

export const messages = {
    "shell.relatedDocuments": {
        text: "Related Documents",
        description: "Label over the cross-links at the foot of a legal page (privacy policy, terms of service).",
    },
    "shell.returnHome": {
        text: "Return Home",
        description: "Button at the foot of a legal page that goes back to the landing page. Title Case here; the privacy page's own footer button uses sentence case.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
