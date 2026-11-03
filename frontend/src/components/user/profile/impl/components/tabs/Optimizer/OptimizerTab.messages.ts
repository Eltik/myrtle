import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "user";

export const messages = {
    "profile.optimizer.noBaseData": {
        text: "This profile has no base data to plan against yet.",
        description: "Shown in place of the base optimizer for an account whose base has never been synced.",
    },
    "profile.optimizer.loadFailed": {
        text: "Couldn't load this base.",
        description: "Shown in place of the base optimizer when the base layout or the room catalog failed to load. A Try again button follows.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
