import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tools";

export const messages = {
    "recruit.empty.noSelection.desc": {
        text: "Combinations appear here once you pick a tag.",
        description: "Empty-state line before any recruitment tag has been picked. The page intro above already explains the tags and the ranking, so this only says where results land.",
    },
    "recruit.empty.noResults.desc": {
        text: "No recruitable operators match the current tags and options. Try removing a tag or enabling more options.",
        description: "Empty-state body when the picked tags yield nothing. 'Options' refers to the Options card beside the results.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
