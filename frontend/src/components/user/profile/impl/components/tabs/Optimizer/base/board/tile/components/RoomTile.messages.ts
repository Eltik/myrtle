import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "user";

export const messages = {
    "profile.base.tile.notBuilt": {
        text: "Not Built",
        description: "Tag on a board slot whose facility the account has not built yet.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
