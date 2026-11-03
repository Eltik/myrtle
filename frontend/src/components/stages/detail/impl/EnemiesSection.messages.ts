import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "stages";

export const messages = {
    "enemies.title": {
        text: "Enemies · {count}",
        description: "Section heading over the grid of enemy cards, with how many distinct enemies the stage fields.",
    },
    "enemies.showOnMap": {
        text: "Show {name} on map",
        description: "Accessible name of an enemy card / spawn row, which scrolls the map to that enemy's route. {name} is the enemy's name from the game data. Also used by the spawn schedule.",
    },
    "enemies.stats": {
        text: "Stats",
        description: "Small button on each enemy card on a stage page. Opens the enemy's stats as they are in this stage (HP, ATK, DEF, RES, speed, attack time), the same card the map shows on hover.",
    },
    "enemies.stats.aria": {
        text: "Stats for {name} in this stage",
        description: "Accessible name of the Stats button on an enemy card. {name} is the enemy's name from the game data. Keep the word for 'Stats' from the visible button text in it.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
