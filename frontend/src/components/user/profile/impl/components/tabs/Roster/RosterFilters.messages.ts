import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

/**
 * The ownership options are a plain table handed to the shared `TagRow`, so
 * they hold message KEYS and this component resolves them with `t()`.
 */
export const namespace = "user";

export const messages = {
    "profile.roster.filters.aria": {
        text: "Roster filters",
        description: "Accessible name of the roster filter panel.",
    },
    "profile.roster.filters.ownership": {
        text: "Ownership",
        description: "Label over the owned / unowned / all choice in the roster filters.",
    },
    "profile.roster.filters.owned": {
        text: "Owned",
        description: "Ownership filter: only operators the account has.",
    },
    "profile.roster.filters.unowned": {
        text: "Unowned",
        description: "Ownership filter: only operators the account lacks.",
    },
    "profile.roster.filters.all": {
        text: "All",
        description: "Ownership filter: every operator, owned or not.",
    },
    "profile.roster.filters.source": {
        text: "Source",
        description: "Label over the any / headhunting / welfare choice in the roster filters: how the operator is obtained.",
    },
    "profile.roster.filters.source.any": {
        text: "Any",
        description: "Source filter: operators from every source.",
    },
    "profile.roster.filters.source.headhunting": {
        text: "Headhunting",
        description: "Source filter: only operators pulled from the gacha (headhunting or recruitment) pool.",
    },
    "profile.roster.filters.source.welfare": {
        text: "Welfare",
        description: "Source filter: only operators the game gives away (event rewards, voucher exchange, Integrated Strategies, story), which all reach max potential for free.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages, dynamic: true });
