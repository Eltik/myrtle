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
    "score.improvements.roguelike.endings.special.label": {
        text: "About the endings count",
        description: "Accessible name of the small (i) button beside a roguelike theme's endings count, which opens a note explaining an ending the count includes.",
    },
    "score.improvements.roguelike.endings.special.item": {
        text: "{name} comes from {mode}.",
        description: "Note beside the endings count. {name} is an ending's in-game name, e.g. 'The Fleetest of Light'; {mode} is a game mode's in-game name, e.g. 'Deep Investigation'. Both come from game data and stay as the game writes them.",
    },
    "score.improvements.roguelike.endings.special.itemNoMode": {
        text: "{name} is not reached in a regular run.",
        description: "Note beside the endings count, used when the game data names no mode for the ending. {name} is an ending's in-game name.",
    },
    "score.improvements.roguelike.endings.special.count": {
        text: "The game data lists {special, plural, one {it} other {them}} as {special, plural, one {an ending} other {endings}}, so this counts {total}: {regular} from regular runs plus {special}.",
        description: "Follows the ending note(s) beside the endings count. {total} is the count shown, {regular} the endings a regular run can reach, {special} the extra ones named before this sentence.",
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
