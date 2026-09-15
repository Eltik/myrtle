import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tools";

export const messages = {
    "birthdays.today.kicker": {
        text: "Today",
        description: "Small heading on the card highlighting the operators whose birthday is today.",
    },
    "birthdays.today.rarityClass": {
        text: "{rarity}★ · {class}",
        description: "Second line of an operator chip: star rating then class, e.g. '6★ · Guard'. {class} is game vocabulary and comes from the game data.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
