import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "user";

export const messages = {
    "leaderboard.toolbar.scope.global": {
        text: "Global",
        description: "Segmented-control option: rank against every public profile.",
    },
    "leaderboard.toolbar.scope.friends": {
        text: "Friends",
        description: "Segmented-control option: rank against your friends only. Disabled for now.",
    },
    "leaderboard.toolbar.comingSoon": {
        text: "Coming soon",
        description: "Tooltip on the disabled Friends option.",
    },
    "leaderboard.toolbar.serverLabel": {
        text: "Change server filter",
        description: "Accessible name of the menu button that filters the leaderboard by game server.",
    },
    "leaderboard.toolbar.server": {
        text: "Server",
        description: "Small uppercase label on the server-filter button, before the current choice.",
    },
    "leaderboard.toolbar.allServers": {
        text: "All servers",
        description: "Menu option that clears the server filter.",
    },
    "leaderboard.toolbar.all": {
        text: "All",
        description: "The server-filter button's value when no server is selected. Sits in a narrow button, so keep it short.",
    },
    "leaderboard.toolbar.intervalLabel": {
        text: "Change movement interval",
        description: "Accessible name of the menu button that picks the window rank movement is measured over.",
    },
    "leaderboard.toolbar.interval": {
        text: "Interval",
        description: "Small uppercase label on the movement-window button, before the current choice.",
    },
    "leaderboard.toolbar.movementOnly": {
        text: "Movement only",
        description: "Label of the toggle that hides players whose rank has not changed.",
    },
    "leaderboard.toolbar.movementOnly.on": {
        text: "Showing only Doctors with movement",
        description: "Tooltip on the movement-only toggle while it is on, describing the current state.",
    },
    "leaderboard.toolbar.movementOnly.off": {
        text: "Show only Doctors with movement",
        description: "Tooltip on the movement-only toggle while it is off, describing what clicking will do.",
    },
    "leaderboard.toolbar.search.placeholder": {
        text: "Search Doctor name or UID…",
        description: "Placeholder in the leaderboard's filter box. 'UID' is the in-game account number. Keep the single-character ellipsis.",
    },
    "leaderboard.toolbar.search.label": {
        text: "Search this page",
        description: "Accessible name of the leaderboard's filter box, which shows only a magnifier icon.",
    },
    "leaderboard.toolbar.search.clear": {
        text: "Clear search",
        description: "Accessible name of the small x button that empties the leaderboard's filter box.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
