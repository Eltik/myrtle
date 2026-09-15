import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "user";

export const messages = {
    "score.improvements.stage.permanent": {
        text: "Permanent",
        description: "Heading of the always-available operations pool.",
    },
    "score.improvements.stage.event": {
        text: "Event",
        description: "Heading of the limited-time operations pool.",
    },
    "score.improvements.stage.threeStarCount": {
        text: "{done} / {total} 3★",
        description: "Count badge on a pool heading: how many of its operations are cleared with three stars. '3★' is the game's own rating.",
    },
    "score.improvements.stage.cleared": {
        text: "Cleared",
        description: "Progress label: operations cleared at all.",
    },
    "score.improvements.stage.threeStarred": {
        text: "3-starred",
        description: "Progress label: operations cleared with the game's full three-star rating.",
    },
    "score.improvements.stage.notThreeStarred": {
        text: "Not 3-starred",
        description: "Progress label: operations cleared but not at three stars.",
    },
    "score.improvements.stage.missing": {
        text: "Missing",
        description: "Heading of the list of operations never cleared.",
    },
    "score.improvements.stage.missing.empty": {
        text: "All stages cleared.",
        description: "Shown in place of the Missing list when nothing is left uncleared.",
    },
    "score.improvements.stage.notThree.title": {
        text: "Cleared, not 3★",
        description: "Heading of the list of operations cleared below three stars. '3★' is the game's own rating.",
    },
    "score.improvements.stage.notThree.subtitle": {
        text: "Run again with a sharper squad to hit 3-star.",
        description: "Blurb under that heading. A 'squad' is the in-game team taken into an operation.",
    },
    "score.improvements.stage.notThree.empty": {
        text: "Every clear is 3★.",
        description: "Shown in place of that list when every clear already has three stars.",
    },
    "score.improvements.stage.annihilation.count": {
        text: "{n} to do now",
        description: "Count badge on the Annihilation heading: maps that can be worked on right now.",
    },
    "score.improvements.stage.annihilation.available": {
        text: "Available now",
        description: "Heading of the Annihilation maps currently playable.",
    },
    "score.improvements.stage.annihilation.availableSubtitle": {
        text: "The three permanent maps and the current weekly rotation. State 2 means cleared but enemy-kill count is below the cap; reaching the cap unlocks the full Orundum reward.",
        description: "Blurb under 'Available now'. 'State 2' is the API's clear state and 'Orundum' an in-game currency - keep the game's own name.",
    },
    "score.improvements.stage.annihilation.maxed": {
        text: "Every currently-playable Annihilation map is maxed.",
        description: "Empty state when no playable Annihilation map has room left. 'Annihilation' is the game mode's own name.",
    },
    "score.improvements.stage.annihilation.locked": {
        text: "Not in rotation",
        description: "Heading of the Annihilation maps that have rotated out of the weekly schedule.",
    },
    "score.improvements.stage.annihilation.lockedSubtitle": {
        text: "Rotating maps that aren't playable right now, so they can't be maxed until their rotation returns. These are scored on a recency curve rather than as permanent gaps - shown for reference.",
        description: "Blurb under 'Not in rotation'.",
    },
    "score.improvements.stage.total": {
        text: "{n} total",
        description: "Count beside an operation list's heading.",
    },
    "score.improvements.stage.chip.named": {
        text: "{code} - {name}",
        description: "Native tooltip on an operation chip: the operation's code and its name, both from the game data.",
    },
    "score.improvements.stage.chip.lockedTitle": {
        text: "{label} - rotated out, not currently playable",
        description: "Native tooltip on a rotated-out Annihilation chip. {label} is the operation's code and name.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
