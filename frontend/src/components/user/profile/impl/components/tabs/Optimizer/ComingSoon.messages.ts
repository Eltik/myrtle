import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "user";

export const messages = {
    "profile.optimizer.comingSoon": {
        text: "{name} is not built yet.",
        description: "Placeholder for an optimizer that has no implementation yet. {name} is the optimizer's name.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
