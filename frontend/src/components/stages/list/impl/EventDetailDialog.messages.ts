import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "stages";

export const messages = {
    "event.stat.operations": {
        text: "Operations",
        description: "Stat tile label in the event detail dialog: how many operations the event holds. 'Operation' is the game's word for a stage.",
    },
    "event.stat.bosses": {
        text: "Bosses",
        description: "Stat tile label in the event detail dialog: how many of its operations contain a boss.",
    },
    "event.stat.codes": {
        text: "Codes",
        description: "Stat tile label in the event detail dialog; its value is the range of stage codes, e.g. '1-1 ~ 1-12'.",
    },
    "event.stat.sanity": {
        text: "Sanity total",
        description: "Stat tile label in the event detail dialog: the summed entry cost of every operation. 'Sanity' is the game's own stamina currency.",
    },
    "event.stat.sanity.free": {
        text: "Free",
        description: "Value of the Sanity total tile when none of the event's operations cost anything to enter.",
    },
    "event.previews": {
        text: "Field previews",
        description: "Heading over the row of small map-preview thumbnails in the event detail dialog.",
    },
    "event.bossInside": {
        text: "{count, plural, one {Boss operation inside} other {Boss operations inside}}",
        description: "Note in the event detail dialog's footer, after a skull icon, when the event contains boss operations.",
    },
    "event.browse": {
        text: "Browse stages",
        description: "Primary button of the event detail dialog: close it and expand that event's row in the list behind.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
