import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "user";

export const messages = {
    "profile.roster.detailed.openAria": {
        text: "Open details for {name}",
        description: "Accessible name of an operator card, which opens that operator's dialog. {name} comes from the game data.",
    },
    "profile.roster.detailed.currentPotential": {
        text: "Current Potential",
        description: "Row label inside the card's Potential section, over the rank the operator has reached.",
    },
    "profile.roster.detailed.loadingSkills": {
        text: "Loading skills…",
        description: "Placeholder while the operator's skill data is still being fetched. Ends with an ellipsis character.",
    },
    "profile.roster.detailed.loadingModules": {
        text: "Loading modules…",
        description: "Placeholder while the operator's module data is still being fetched. Ends with an ellipsis character.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
