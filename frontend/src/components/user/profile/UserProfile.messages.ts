import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "user";

export const messages = {
    "profile.tab.stats": {
        text: "Stats",
        description: "Profile tab: collection and account statistics.",
    },
    "profile.tab.score": {
        text: "Score",
        description: "Profile tab: the account's graded score breakdown.",
    },
    "profile.tab.roster": {
        text: "Roster",
        description: "Profile tab: the operators this account owns.",
    },
    "profile.tab.plans": {
        text: "Plans",
        description: "Profile tab: the account's public upgrade plans.",
    },
    "profile.tab.inventory": {
        text: "Inventory",
        description: "Profile tab: the items this account holds.",
    },
    "profile.tab.enemies": {
        text: "Enemies",
        description: "Profile tab: which enemies the account has encountered.",
    },
    "profile.tab.optimizer": {
        text: "Optimizer",
        description: "Profile tab: tools that suggest improvements to the account.",
    },
    "profile.notFound.eyebrow": {
        text: "Player",
        description: "Small uppercase label above the 'profile not found' heading. 'Player' is the site's term; the game itself says 'Doctor'.",
    },
    "profile.notFound.title": {
        text: "Profile not found",
        description: "Heading shown when the requested profile does not exist or is private.",
    },
    "profile.notFound.desc": {
        text: "No player with ID {id} exists, or their profile is private.",
        description: "Body of the 'profile not found' page. {id} is the account ID, rendered in monospace, and may move wherever the sentence needs it. 'Player' is the site's term; the game itself says 'Doctor'.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
