import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tools";

export const messages = {
    "randomizer.preview.stageLabel": {
        text: "{code} - {name}",
        description: "Names a stage by its code and title, e.g. '7-18 - Cold Light'. Both halves come from the game data; the dash is a plain hyphen.",
    },
    "randomizer.preview.expand": {
        text: "Expand {stage} map preview",
        description: "Accessible name of the thumbnail that opens the full stage map. {stage} is the stage's code and title.",
    },
    "randomizer.preview.none": {
        text: "No preview available for {code}",
        description: "Accessible name of the thumbnail when no map image exists, and the screen-reader text inside the placeholder. {code} is the stage's code.",
    },
    "randomizer.preview.alt": {
        text: "{stage} map preview",
        description: "Alt text of the stage map image. {stage} is the stage's code and title.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
