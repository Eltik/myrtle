import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "user";

export const messages = {
    "leaderboard.itemYou.title": {
        text: "You",
        description: "Heading of the signed-in player's own standing card while the table is ranked by an item.",
    },
    "leaderboard.itemYou.rank": {
        text: "Rank",
        description: "Label before the player's global rank for the item, as in 'Rank #12'.",
    },
    "leaderboard.itemYou.topPercentile": {
        text: " · Top ",
        description: "Separator and label before the percentile, as in 'Rank #12 · Top 3.4%'. Keep the surrounding spaces.",
    },
    "leaderboard.itemYou.metric.held": {
        text: "Held",
        description: "Label over how much of the ranked item the player holds.",
    },
    "leaderboard.itemYou.metric.server": {
        text: "On {server}",
        description: "Label over the player's rank among holders on their own server. {server} is the server code, e.g. EN.",
    },
    "leaderboard.itemYou.metric.holders": {
        text: "Holders",
        description: "Label over how many visible players hold the item at all.",
    },
    "leaderboard.itemYou.none": {
        text: "You hold none of this item, so you are not ranked for it.",
        description: "Shown when the signed-in player has a public profile but holds none of the ranked item.",
    },
    "leaderboard.itemYou.viewProfile": {
        text: "View profile",
        description: "Button linking to the signed-in player's own profile.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
