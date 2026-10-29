import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "user";

export const messages = {
    "score.improvements.sandbox.empty": {
        text: "No RA progress detected. Play Reclamation Algorithm to begin tracking.",
        description: "Empty state in the RA panel. 'RA' abbreviates the game mode Reclamation Algorithm and keeps the game's own name.",
    },
    "score.improvements.sandbox.topicShare": {
        text: "{pct}% · {share}% of RA",
        description: "Badge beside one Reclamation Algorithm topic (a season of the mode, such as 'Tales Within the Sand'): the player's completion of that topic, then how much of the overall RA score the topic carries. Every topic carries the same share. 'RA' is the game mode's own abbreviation.",
    },
    "score.improvements.sandbox.weight": {
        text: "{pct}% of RA",
        description: "Pill beside a Reclamation Algorithm category saying how much of the RA score it carries. 'RA' is the game mode's own abbreviation.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
