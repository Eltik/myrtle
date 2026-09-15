import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "stages";

export const messages = {
    "mapSettings.title": {
        text: "Settings",
        description: "Kicker over the map's layer toggles.",
    },
    "mapSettings.showRoutes": {
        text: "Route Lines",
        description: "Map layer toggle: draw the enemy route lines.",
    },
    "mapSettings.showRoutes.desc": {
        text: "Draw the paths enemies follow across the map.",
        description: "Caption under the Route Lines toggle.",
    },
    "mapSettings.showEnemyIcons": {
        text: "Enemy Icons",
        description: "Map layer toggle: show each spawning enemy's icon on its route.",
    },
    "mapSettings.showEnemyIcons.desc": {
        text: "Show each spawning enemy's icon on its route.",
        description: "Caption under the Enemy Icons toggle.",
    },
    "mapSettings.showMovement": {
        text: "Enemy Movement",
        description: "Map layer toggle: animate the enemy icon along its route.",
    },
    "mapSettings.showMovement.desc": {
        text: "Animate the icon along its route instead of parking it at the spawn.",
        description: "Caption under the Enemy Movement toggle.",
    },
    "mapSettings.showTimers": {
        text: "Wait Timers",
        description: "Map layer toggle: show the wait-duration badges on routes.",
    },
    "mapSettings.showTimers.desc": {
        text: "Show how long enemies pause at points on their route.",
        description: "Caption under the Wait Timers toggle.",
    },
    "mapSettings.walkingChibis": {
        text: "Walking Chibis",
        description: "Map layer toggle: render the enemy's animated chibi walking its route. 'Chibi' is the game's small animated sprite.",
    },
    "mapSettings.walkingChibis.desc": {
        text: "In 3D view, replace the icon with the enemy's animated chibi walking the route.",
        description: "Caption under the Walking Chibis toggle.",
    },
} satisfies MessageMap;

// `dynamic`: the per-toggle keys live on the module-level SETTINGS table and
// are resolved as `t(def.labelKey)`, so the extractor has no literal call site.
export const { keys } = defineMessages({ namespace, messages, dynamic: true });
