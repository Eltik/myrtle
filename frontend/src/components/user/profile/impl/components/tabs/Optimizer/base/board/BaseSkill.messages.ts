import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "user";

export const messages = {
    "profile.base.skill.locked": {
        text: "E{elite} Lv{level}",
        description: "Tag beside a base skill the operator has not unlocked yet: the promotion and level it needs. 'E' and 'Lv' are the game's own abbreviations. Rendered uppercase by CSS.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
