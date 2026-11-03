import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "gacha";

export const messages = {
    "history.error.names": {
        text: "Operator or banner names couldn't be loaded, so some pulls show without them.",
        description: "Notice above the pull-history panels when the operator list or the banner list fails to load; the pulls still count, but rows lose their operator or banner names.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
