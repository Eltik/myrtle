import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "user";

export const messages = {
    "profile.strip.operators.kicker": {
        text: "Operators",
        description: "Label of the tile counting the operators this account owns.",
    },
    "profile.strip.skins.kicker": {
        text: "Skins",
        description: "Label of the tile counting the outfits this account owns.",
    },
    "profile.strip.items.kicker": {
        text: "Items",
        description: "Label of the tile counting the distinct items in this account's inventory.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
