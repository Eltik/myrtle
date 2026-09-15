import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tools";

export const messages = {
    "release.controls.stageOnly": {
        text: "Stage events only",
        description: "Switch label: show only events that come with combat stages, not pure-story ones.",
    },
    "release.controls.dotLegend": {
        text: "Filled dot: confirmed. Hollow dot: estimated.",
        description: "Legend explaining the two dot styles on the schedule pills.",
    },
    "release.anchor.title": {
        text: "Released on CN the day {event} started; 96% of such skins reach EN with their event",
        description: "Tooltip explaining why a skin is expected alongside an event. {event} is the event's own id; 'CN' and 'EN' are the Chinese and English game servers.",
    },
    "release.anchor.with": {
        text: "with {event}",
        description: "Names the event a row arrives alongside, e.g. 'with Ideal City'. {event} is that event's name and may move wherever the phrase needs it.",
    },
    "release.detail.close": {
        text: "Close",
        description: "Accessible name of the cross that closes the expanded schedule row.",
    },
    "release.detail.cnRelisted": {
        text: "CN re-listed",
        description: "Small heading before the dates a rerun was put back on sale in China. 'CN' is the Chinese game server and stays as-is.",
    },
    "release.detail.cn": {
        text: "CN",
        description: "Small heading before the Chinese-server dates of a row. The game server's usual abbreviation.",
    },
    "release.farm.stageTitle": {
        text: "{code}: {ap} sanity",
        description: "Tooltip on a farming stage: its code and what it costs to run. 'Sanity' is the game's own name for the energy a stage spends.",
    },
    "release.farm.ap": {
        text: "{ap} AP",
        description: "A stage's energy cost, e.g. '21 AP'. 'AP' is the game's own abbreviation for sanity.",
    },
    "release.farm.dropTitle": {
        text: "{item}: {rate}",
        description: "Tooltip on a drop icon: the item's name then how often it drops.",
    },
    "release.occ.always": {
        text: "guaranteed",
        description: "Drop rate: the item always drops. Lowercase; it sits under the item name.",
    },
    "release.occ.almost": {
        text: "almost always",
        description: "Drop rate: the item nearly always drops. Lowercase; it sits under the item name.",
    },
    "release.occ.usual": {
        text: "usual",
        description: "Drop rate: the item drops more often than not. Lowercase; it sits under the item name.",
    },
    "release.occ.often": {
        text: "often",
        description: "Drop rate: the item drops fairly often. Lowercase; it sits under the item name.",
    },
    "release.occ.sometimes": {
        text: "sometimes",
        description: "Drop rate: the item drops now and then. Lowercase; it sits under the item name.",
    },
    "release.occ.rarely": {
        text: "rare",
        description: "Drop rate: the item drops seldom. Lowercase; it sits under the item name.",
    },
} satisfies MessageMap;

// `dynamic`: the drop-rate keys are stored in a lookup table and resolved as
// `t(OCC_LABEL_KEYS[occ])`, so the
// extractor has no literal call site to match them against.
export const { keys } = defineMessages({ namespace, messages, dynamic: true });
