import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "home";

export const messages = {
    "leaders.eyebrow": {
        text: "Leaderboard",
        description: "Small uppercase label above the home page's top-five player panel.",
    },
    "leaders.title": {
        text: "Top Doctors",
        description: "Heading of the home page's top-five player panel. 'Doctor' is what the game calls the player; use the game's own term.",
    },
    "leaders.seeAll": {
        text: "Full leaderboard →",
        description: "Link from the home page's top-five panel to the full leaderboard. The arrow is part of the label.",
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
    "leaders.viewProfile": {
        text: "View {nickname}'s profile",
        description: "Accessible name of a player row on the home page leaderboard, which opens that player's profile.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
