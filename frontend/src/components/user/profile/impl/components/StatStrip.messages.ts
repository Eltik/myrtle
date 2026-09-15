import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "user";

export const messages = {
    "profile.strip.operators.kicker": {
        text: "Operators",
        description: "Label of the tile counting the operators this account owns.",
    },
    "profile.strip.operators.sub": {
        text: "Unique units in roster",
        description: "Caption under the operator count.",
    },
    "profile.strip.skins.kicker": {
        text: "Skins",
        description: "Label of the tile counting the outfits this account owns.",
    },
    "profile.strip.skins.sub": {
        text: "Outfits collected",
        description: "Caption under the skin count.",
    },
    "profile.strip.items.kicker": {
        text: "Items",
        description: "Label of the tile counting the distinct items in this account's inventory.",
    },
    "profile.strip.items.sub": {
        text: "Inventory entries",
        description: "Caption under the item count.",
    },
    "profile.strip.orundum.sub": {
        text: "Orundum Available",
        description: "Caption under the Orundum balance. 'Orundum' is an in-game currency; keep the game's own name for it.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
