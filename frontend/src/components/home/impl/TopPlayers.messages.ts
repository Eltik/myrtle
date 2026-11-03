import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "home";

export const messages = {
    "leaders.title": {
        text: "Top players",
        description: "Heading of the home page's top-five player panel. 'Player' is the site's term; the game itself says 'Doctor'.",
    },
    "leaders.seeAll": {
        text: "Full leaderboard",
        description: "Link from the home page's top-five panel to the full leaderboard.",
    },
    "leaders.meta": {
        text: "{server} · Lv {level}",
        description: "Second line of a player row on the home page leaderboard. {server} is a game server code such as 'EN' or 'CN'; {level} is the player's account level.",
    },
    "leaders.empty": {
        text: "No ranked players yet.",
        description: "Shown in the home page's top-five panel when the leaderboard is empty.",
    },
    "leaders.error": {
        text: "Couldn't load the leaderboard.",
        description: "Shown in the home page's top-five panel when the leaderboard failed to load.",
    },
    "leaders.retry": {
        text: "Try again",
        description: "Button beside the error message in the home page's top-five panel; it reloads the leaderboard.",
    },
    "leaders.loading": {
        text: "Loading the leaderboard…",
        description: "Read out by screen readers only, while the home page's top-five panel loads.",
    },
    "leaders.missing": {
        text: "-",
        description: "Shown in a home page leaderboard row in place of a rank, score or grade the player does not have. Keep it a short placeholder mark.",
    },
    "leaders.missingLabel": {
        text: "Not available",
        description: "Read out by screen readers only, in place of the placeholder mark for a rank, score or grade the player does not have on the home page leaderboard.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
