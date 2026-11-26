import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "user";

export const messages = {
    "profile.stats.rosterPrivate": {
        text: "This player keeps their roster private, so the statistics drawn from it are hidden too.",
        description: "Note on another player's Stats tab when that player hid their Roster tab. Only the sign-in cards remain.",
    },
    "profile.stats.error": {
        text: "Couldn't load the statistics.",
        description: "Shown in place of the Stats tab's cards when the skin list they are counted against fails to load.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
