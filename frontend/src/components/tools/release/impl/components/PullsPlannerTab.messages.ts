import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tools";

export const messages = {
    "release.pulls.title": {
        text: "Pulls planner is not built yet.",
        description: "Placeholder heading on the unfinished Pulls planner tab. A 'pull' is one gacha draw.",
    },
    "release.pulls.desc": {
        text: "This will be a feature very soon!",
        description: "Placeholder body on the unfinished Pulls planner tab.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
