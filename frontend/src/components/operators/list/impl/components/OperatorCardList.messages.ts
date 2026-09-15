import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "operators";

export const messages = {
    "card.rarityStars": {
        text: "{rarity} star",
        description: "Accessible name of the star row on a list-view card. Always singular in English on purpose: it reads as a rating name ('4 star'), not a count of stars.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
