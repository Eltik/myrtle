import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "gacha";

export const messages = {
    "community.leaderboard.kicker": {
        text: "Most pulled · per rarity",
        description: "Heading of the most-pulled operators panel. Keep the middle dot.",
    },
    "community.leaderboard.empty": {
        text: "No data for this rarity yet.",
        description: "Empty state when nobody has shared a pull of the selected rarity.",
    },
    "community.leaderboard.rarityFilter": {
        text: "Rarity",
        description: "Accessible name of the 6★ / 5★ / 4★ switch that picks which rarity the most-pulled table lists.",
    },
    "community.leaderboard.col.operator": {
        text: "Operator",
        description: "Column heading for the operator's name. Rendered uppercase.",
    },
    "community.leaderboard.col.pulls": {
        text: "Pulls",
        description: "Column heading for how many times the community pulled that operator. Rendered uppercase in a narrow column.",
    },
    "community.leaderboard.col.share": {
        text: "Share",
        description: "Column heading for that operator's percentage of all pulls of its rarity. Rendered uppercase in a narrow column.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
