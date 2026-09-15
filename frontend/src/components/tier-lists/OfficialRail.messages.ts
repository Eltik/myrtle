import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tierLists";

export const messages = {
    "browse.official.title": {
        text: "Official",
        description: "Heading of the row of tier lists maintained by the site's own team, as opposed to community ones.",
    },
    "browse.official.viewAll": {
        text: "View all",
        description: "Link that filters the grid below to every official tier list.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
