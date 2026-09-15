import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "changelog";

export const messages = {
    "list.empty": {
        text: "No release notes written yet. The Commits tab has the raw history.",
        description: "Empty state on the Release notes tab. 'Commits' names the sibling tab and should match its label.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
