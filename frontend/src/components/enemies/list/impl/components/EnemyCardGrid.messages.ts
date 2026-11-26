import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "enemies";

export const messages = {
    "cardGrid.hp": {
        text: "HP",
        description: "Stat bar label on an enemy grid card: hit points. Fixed 24px column, so it must stay this short.",
    },
    "cardGrid.atk": {
        text: "ATK",
        description: "Stat bar label on an enemy grid card: attack. Fixed 24px column, so it must stay this short.",
    },
    "cardGrid.def": {
        text: "DEF",
        description: "Stat bar label on an enemy grid card: defense. Fixed 24px column, so it must stay this short.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
