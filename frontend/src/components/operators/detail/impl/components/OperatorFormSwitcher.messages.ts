import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "operators";

export const messages = {
    "forms.aria": {
        text: "Operator form",
        description: "Accessible name of the switcher between an operator's alternate forms (e.g. Amiya's Caster and Guard). The form names themselves are class names from the game data.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
