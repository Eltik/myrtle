import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tierLists";

export const messages = {
    "browse.hero.title": {
        text: "Tier lists",
        description: "Heading of the tier-list browse page. 'The meta' is the community's current consensus on what is strong.",
    },
    "browse.hero.blurb": {
        text: "{count, plural, one {list} other {lists}} from the team and the community. Sort and filter by type, flair or author.",
        description: "Sentence under the browse heading; the count itself is rendered just before it, so this starts with the noun it counts.",
    },
    "browse.hero.blurbEmpty": {
        text: "Official lists from the team and lists from the community. Sort and filter by type, flair or author.",
        description: "Sentence under the browse heading while no lists have loaded, so there is no count to lead with.",
    },
    "browse.hero.create": {
        text: "Create list",
        description: "Button taking a signed-in player to their own tier lists, where a new one can be made.",
    },
    "browse.hero.signIn": {
        text: "Sign in to publish",
        description: "Button shown instead of 'Create list' to a signed-out visitor; it opens the sign-in dialog.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
