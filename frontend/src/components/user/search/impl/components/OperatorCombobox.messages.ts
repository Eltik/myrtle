import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "user";

export const messages = {
    "search.operatorPicker.loading": {
        text: "Loading operators…",
        description: "Placeholder in an operator picker while the operator index loads. Keep the single-character ellipsis.",
    },
    "search.operatorPicker.empty": {
        text: "No matching operators.",
        description: "Shown in an operator picker's list when the typed text matches nobody.",
    },
    "search.operatorPicker.rarity": {
        text: "{rarity}★",
        description: "Rarity shown under an operator's name in the picker list, before the class. Keep the star character.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
