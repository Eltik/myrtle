import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "gacha";

export const messages = {
    "history.breakdown.kicker": {
        text: "Banner breakdown",
        description: "Small uppercase label above the table of the player's pulls grouped by banner.",
    },
    "history.breakdown.title": {
        text: "Where your pulls went.",
        description: "Heading of the panel grouping the player's pulls by banner.",
    },
    "history.breakdown.empty": {
        text: "No banner data yet.",
        description: "Empty state when the player has no pulls to group by banner.",
    },
    "history.breakdown.col.banner": {
        text: "Banner",
        description: "Column heading for the banner's name. Rendered uppercase.",
    },
    "history.breakdown.col.lastPull": {
        text: "Last pull",
        description: "Column heading for the date of the player's most recent pull on that banner. Rendered uppercase.",
    },
    "history.breakdown.col.pulls": {
        text: "Pulls",
        description: "Column heading for how many pulls the player made on that banner. Rendered uppercase.",
    },
    "history.breakdown.col.sixStars": {
        text: "6★",
        description: "Column heading for how many 6-star operators that banner gave the player. Rendered uppercase in a very narrow column.",
    },
    "history.breakdown.col.distribution": {
        text: "Distribution",
        description: "Column heading over the bar comparing each banner's pull count with the largest. Rendered uppercase.",
    },
    "history.breakdown.stat.pulls": {
        text: "Pulls",
        description: "Label over the pull count in a banner's hover card. Rendered uppercase.",
    },
    "history.breakdown.stat.sixStars": {
        text: "6★",
        description: "Label over the 6-star count in a banner's hover card. Rendered uppercase.",
    },
    "history.breakdown.stat.fiveStars": {
        text: "5★",
        description: "Label over the 5-star count in a banner's hover card. Rendered uppercase.",
    },
    "history.breakdown.featured": {
        text: "Featured {rarity}★",
        description: "Label over the row of operators a banner boosts, e.g. 'Featured 6★'. Rendered uppercase.",
    },
    "history.breakdown.moreOperators": {
        text: "{count} more",
        description: "Tooltip on the '+3' chip that stands in for featured operators the card has no room to show.",
    },
    "history.breakdown.guarantee": {
        text: "{name} within {count} pulls",
        description: "A banner's guarantee, e.g. 'Guaranteed 5★ within 10 pulls'. {name} is the guarantee's own name from the game data where it has one.",
    },
    "history.breakdown.guaranteeDefault": {
        text: "Guaranteed 5★",
        description: "Stand-in name for a banner's guarantee when the game data leaves it blank.",
    },
    "history.breakdown.dateRange": {
        text: "{open} → {close}",
        description: "A banner's run window in its hover card, e.g. 'Mar 3, 2025 → Mar 17, 2025'. Keep the arrow.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
