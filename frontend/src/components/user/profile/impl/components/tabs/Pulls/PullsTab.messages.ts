import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "user";

export const messages = {
    "profile.pulls.private": {
        text: "This player's pull history is private.",
        description: "Shown on another player's Pulls tab when their pull history cannot be read (they made it private since the page loaded).",
    },
    "profile.pulls.empty": {
        text: "No pulls recorded yet.",
        description: "Shown on a Pulls tab when the player has no stored pulls.",
    },
    "profile.pulls.emptyFiltered": {
        text: "No pulls on the chosen banners.",
        description: "Shown on a Pulls tab when the banner filter matches none of the player's pulls.",
    },
    "profile.pulls.ownerPrivate": {
        text: "Only you can see this tab. Visitors see it once your pull history is public.",
        description: "Note on the owner's own Pulls tab while their pull history is private, the default.",
    },
    "profile.pulls.ownerEmpty": {
        text: "Sync your pulls on the history page to fill this tab.",
        description: "Shown on the owner's own Pulls tab when they have no stored pulls; followed by a link to the pull-history page.",
    },
    "profile.pulls.historyLink": {
        text: "Open pull history",
        description: "Link from the owner's Pulls tab to their full pull-history page, where pulls are synced.",
    },
    "profile.pulls.loadFailed": {
        text: "Couldn't load the pull history.",
        description: "Shown on a Pulls tab when the pulls failed to load; followed by the reason.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
