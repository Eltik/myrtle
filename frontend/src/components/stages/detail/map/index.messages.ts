import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "stages";

export const messages = {
    "map.toggle3d": {
        text: "Toggle 3D view",
        description: "Accessible name of the icon button that tilts the map board into its faux-3D view.",
    },
    "map.prevEnemy": {
        text: "Previous enemy",
        description: "Accessible name of the icon button that steps the map back to the previous enemy spawn.",
    },
    "map.nextEnemy": {
        text: "Next enemy",
        description: "Accessible name of the icon button that steps the map forward to the next enemy spawn.",
    },
    "map.watermark": {
        text: "Map",
        description: "Watermark drawn in the corner of the board when no stage code was passed in. Normally the stage's own code is shown instead, so this is a fallback; keep it very short.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
