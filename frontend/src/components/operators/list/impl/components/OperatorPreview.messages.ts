import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "operators";

export const messages = {
    "preview.position": {
        text: "Position",
        description: "Small key in the hover preview; its value is the deployment position (melee or ranged) from the game data.",
    },
    "preview.race": {
        text: "Race",
        description: "Small key in the hover preview; its value is the operator's race from the game data.",
    },
    "preview.gender": {
        text: "Gender",
        description: "Small key in the hover preview; its value comes from the game data.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
