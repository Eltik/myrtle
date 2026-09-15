import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "user";

export const messages = {
    "profile.base.shift.aria": {
        text: "Rotation shift",
        description: "Accessible name of the buttons that switch between the planned rotation's shifts.",
    },
    "profile.base.shift.now": {
        text: "Stationed now",
        description: "Button showing the crews the account currently has stationed, as opposed to a planned shift.",
    },
    "profile.base.shift.n": {
        text: "Shift {n}",
        description: "Button for one shift of the planned rotation; {n} counts from 1.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
