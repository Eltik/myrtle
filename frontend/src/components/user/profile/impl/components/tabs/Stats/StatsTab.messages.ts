import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "user";

export const messages = {
    "profile.stats.rosterPrivate": {
        text: "This player keeps their roster private, so the statistics drawn from it are hidden too.",
        description: "Note on another player's Stats tab when that player hid their Roster tab. Only the sign-in cards remain.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
