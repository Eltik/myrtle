import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

/**
 * The board tiles' own labels. `tile-labels.ts` is plain `.ts`, so it takes a
 * `t` and returns resolved text; room names themselves come from the base
 * catalog, not from here.
 */
export const namespace = "user";

export const messages = {
    "profile.base.tile.empty": {
        text: "Empty",
        description: "Name of a board slot with no facility built in it.",
    },
    "profile.base.tile.aria": {
        text: "{name}, level {level}, {staffed} of {seats} staffed",
        description: "Accessible name of one board tile. {name} is the room's name, {staffed} and {seats} count operators.",
    },
    "profile.base.tile.crew.added": {
        text: "{name} - joins this shift",
        description: "Tooltip on an operator the planned rotation moves into this room for the shift being viewed.",
    },
    "profile.base.tile.crew.removed": {
        text: "{name} - leaves this shift",
        description: "Tooltip on an operator the planned rotation moves out of this room for the shift being viewed.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
