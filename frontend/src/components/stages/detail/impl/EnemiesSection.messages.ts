import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "stages";

export const messages = {
    "enemies.title": {
        text: "Enemies · {count}",
        description: "Kicker over the grid of enemy cards, with how many distinct enemies the stage fields.",
    },
    "enemies.showOnMap": {
        text: "Show {name} on map",
        description: "Accessible name of an enemy card / spawn row, which scrolls the map to that enemy's route. {name} is the enemy's name from the game data. Also used by the spawn schedule.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
