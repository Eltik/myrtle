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
    "randomizer.preview.zoomOut": {
        text: "Zoom out",
        description: "Accessible name and tooltip of the zoom-out button in the map viewer.",
    },
    "randomizer.preview.zoomIn": {
        text: "Zoom in",
        description: "Accessible name and tooltip of the zoom-in button in the map viewer.",
    },
    "randomizer.preview.resetView": {
        text: "Reset view",
        description: "Accessible name and tooltip of the button that returns the map viewer to its starting zoom and position.",
    },
    "randomizer.preview.download": {
        text: "Download",
        description: "Accessible name and tooltip of the button that saves the map image.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
