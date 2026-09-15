import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "stages";

export const messages = {
    "enemyStats.hp": {
        text: "HP",
        description: "Stat label in the map's enemy hover card: hit points. Abbreviated to fit a very narrow tile.",
    },
    "enemyStats.atk": {
        text: "ATK",
        description: "Stat label in the map's enemy hover card: attack. Abbreviated to fit a very narrow tile.",
    },
    "enemyStats.def": {
        text: "DEF",
        description: "Stat label in the map's enemy hover card: defense. Abbreviated to fit a very narrow tile.",
    },
    "enemyStats.res": {
        text: "RES",
        description: "Stat label in the map's enemy hover card: magic resistance, shown as a percentage. Abbreviated to fit a very narrow tile.",
    },
    "enemyStats.spd": {
        text: "Spd",
        description: "Stat label in the map's enemy hover card: move speed in tiles per second. Abbreviated to fit a very narrow tile.",
    },
    "enemyStats.atkTime": {
        text: "ATK Time",
        description: "Stat label in the map's enemy hover card: seconds between attacks. Abbreviated to fit a very narrow tile.",
    },
    "enemyStats.none": {
        text: "No stat data available.",
        description: "Shown in the map's enemy hover card when the game data ships no stat block for that enemy.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
