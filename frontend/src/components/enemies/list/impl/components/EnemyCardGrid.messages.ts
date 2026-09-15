import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "enemies";

export const messages = {
    "cardGrid.aria": {
        text: "{name} ({index})",
        description: "Accessible name of an enemy card in the grid. {name} is the enemy's name and {index} its in-game handbook callsign, both game data; only the punctuation around them is yours.",
    },
    "cardGrid.portraitAlt": {
        text: "{name} portrait",
        description: "Alt text of an enemy's portrait image on a grid card. {name} is the enemy's name from the game data.",
    },
    "cardGrid.hp": {
        text: "HP",
        description: "Stat bar label on an enemy grid card: hit points. Fixed 20px column, so it must stay this short.",
    },
    "cardGrid.atk": {
        text: "ATK",
        description: "Stat bar label on an enemy grid card: attack. Fixed 20px column, so it must stay this short.",
    },
    "cardGrid.def": {
        text: "DEF",
        description: "Stat bar label on an enemy grid card: defense. Fixed 20px column, so it must stay this short.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
