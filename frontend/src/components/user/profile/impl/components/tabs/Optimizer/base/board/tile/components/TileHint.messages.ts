import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "user";

export const messages = {
    "profile.base.tile.level": {
        text: "Level",
        description: "Row label in a board tile's hover card, over the facility's level.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
