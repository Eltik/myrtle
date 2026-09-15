import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tools";

export const messages = {
    "recruit.empty.noSelection.title": {
        text: "Pick tags to see combinations",
        description: "Empty-state heading before any recruitment tag has been picked.",
    },
    "recruit.empty.noSelection.desc": {
        text: "Select the tags shown in your in-game recruitment screen. Combinations are ranked by guaranteed minimum rarity.",
        description: "Empty-state body before any recruitment tag has been picked.",
    },
    "recruit.empty.noResults.title": {
        text: "No matching operators",
        description: "Empty-state heading when the picked tags yield nothing.",
    },
    "recruit.empty.noResults.desc": {
        text: "No recruitable operators match the current tags and options. Try removing a tag or enabling more options.",
        description: "Empty-state body when the picked tags yield nothing. 'Options' refers to the Options card beside the results.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
