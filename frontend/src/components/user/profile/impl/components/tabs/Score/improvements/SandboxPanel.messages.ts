import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "user";

export const messages = {
    "score.improvements.sandbox.empty": {
        text: "No RA progress detected. Start RA in Operation Originium Dust to begin tracking.",
        description: "Empty state in the RA panel. 'RA' abbreviates the game mode Reclamation Algorithm and 'Operation Originium Dust' is the in-game event that hosts it - both keep the game's own names.",
    },
    "score.improvements.sandbox.weight": {
        text: "{pct}% of RA",
        description: "Pill beside a Reclamation Algorithm category saying how much of the RA score it carries. 'RA' is the game mode's own abbreviation.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
