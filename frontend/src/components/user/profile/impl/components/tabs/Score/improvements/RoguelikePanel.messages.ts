import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "user";

export const messages = {
    "score.improvements.roguelike.empty": {
        text: "No roguelike themes loaded for this account.",
        description: "Empty state in the Roguelike panel. A 'theme' is one Integrated Strategies season.",
    },
    "score.improvements.roguelike.endings": {
        text: "Endings",
        description: "Progress label: how many of a roguelike theme's run endings have been reached.",
    },
    "score.improvements.roguelike.bp": {
        text: "BP levels",
        description: "Progress label for a roguelike theme's battle-pass levels. 'BP' is the in-game abbreviation.",
    },
    "score.improvements.roguelike.difficulty": {
        text: "Difficulty",
        description: "Progress label: the highest difficulty level cleared in a roguelike theme.",
    },
    "score.improvements.roguelike.challenges": {
        text: "Challenges",
        description: "Progress label: a roguelike theme's optional challenge objectives.",
    },
    "score.improvements.roguelike.collectibles": {
        text: "Collectibles",
        description: "Progress label covering every collectible type in a roguelike theme.",
    },
    "score.improvements.roguelike.relics": {
        text: "Relics",
        description: "Caption on one collectible count. 'Relics' are an in-game Integrated Strategies collectible.",
    },
    "score.improvements.roguelike.capsules": {
        text: "Capsules",
        description: "Caption on one collectible count. 'Capsules' are an in-game Integrated Strategies collectible.",
    },
    "score.improvements.roguelike.bands": {
        text: "Bands",
        description: "Caption on one collectible count. 'Bands' are the in-game Integrated Strategies starting squads.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
