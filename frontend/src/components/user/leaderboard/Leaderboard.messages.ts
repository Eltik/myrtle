import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "user";

export const messages = {
    "leaderboard.breadcrumb.label": {
        text: "Breadcrumb",
        description: "Accessible name of the breadcrumb <nav> above the leaderboard.",
    },
    "leaderboard.breadcrumb.doctors": {
        text: "Players",
        description: "First breadcrumb crumb, the section that lists players. 'Player' is the site's term; the game itself says 'Doctor'.",
    },
    "leaderboard.breadcrumb.current": {
        text: "Leaderboard",
        description: "Last breadcrumb crumb, naming the current page.",
    },
    "leaderboard.showing": {
        text: "Showing {start}-{end} of {count, number} {count, plural, one {player} other {players}}",
        description: "Range summary in the table footer. {start} and {end} are the first and last row numbers on this page, {count} the number of players matching the filters.",
    },
    "leaderboard.showingHolders": {
        text: "Showing {start}-{end} of {count, number} {count, plural, one {holder} other {holders}}",
        description: "Range summary in the table footer of the inventory view. {start} and {end} are the first and last row numbers on this page, {count} the number of players holding the item.",
    },
    "leaderboard.loadFailed.players": {
        text: "The leaderboard did not load.",
        description: "Failure line in place of the ranked table when the page of players could not be fetched (network or server error), above a Try again button.",
    },
    "leaderboard.loadFailed.holders": {
        text: "The holders of this item did not load.",
        description: "Failure line in place of the inventory-ranked table when the page of item holders could not be fetched, above a Try again button.",
    },
    "leaderboard.loadFailed.movers": {
        text: "The biggest movers did not load.",
        description: "Failure line in place of the sidebar card listing the players whose rank rose most, beside a Try again button.",
    },
    "leaderboard.loadFailed.items": {
        text: "The item list did not load.",
        description: "Failure line in place of the sidebar card listing the most-held items, when the item catalog or item names failed to load, beside a Try again button.",
    },
    "leaderboard.loadFailed.standing": {
        text: "Your standing did not load.",
        description: "Failure line in place of the sidebar card with the signed-in player's own rank, beside a Try again button.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
