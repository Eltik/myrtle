import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tools";

export const messages = {
    "randomizer.empty.kicker": {
        text: "No squads rolled",
        description: "Small heading over the empty state before anything has been drawn.",
    },
    "randomizer.empty.title": {
        text: "Awaiting randomizer.",
        description: "Large empty-state line, written as a clipped status report. Keep the full stop.",
    },
    "randomizer.empty.hint": {
        text: "Hit {key} above to draw a random stage, a squad of operators, and a challenge modifier.",
        description: "Empty-state hint before anything has been drawn. {key} is the Roll Squad button name in a key-cap box, labelled by randomizer.empty.hintKey, and may move wherever the sentence needs it.",
    },
    "randomizer.empty.hintKey": {
        text: "Roll Squad",
        description: "The button name shown in a key-cap box inside the empty-state hint. It must match the main button's label.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
