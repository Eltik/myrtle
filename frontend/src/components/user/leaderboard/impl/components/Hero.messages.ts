import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "user";

export const messages = {
    "leaderboard.hero.eyebrow": {
        text: "Players · Leaderboard",
        description: "Small uppercase label above the leaderboard heading, naming the section.",
    },
    "leaderboard.hero.title": {
        text: "Top Players",
        description: "Leaderboard page heading. 'Player' is the site's term; the game itself says 'Doctor'.",
    },
    "leaderboard.hero.allTime": {
        text: "All-time",
        description: "Pill beside the leaderboard heading saying the ranking covers a player's whole history rather than a season.",
    },
    "leaderboard.hero.blurb": {
        text: "A composite ranking across roster depth, base efficiency, and combat scoring. Pulled from public profiles only.",
        description: "Paragraph under the leaderboard heading explaining what the ranking measures.",
    },
    "leaderboard.hero.stat.ranked": {
        text: "Players ranked",
        description: "Label under the count of players on the leaderboard. Fits a narrow tile, so keep it short.",
    },
    "leaderboard.hero.stat.topScore": {
        text: "Top score",
        description: "Label under the highest total score on the leaderboard. Fits a narrow tile, so keep it short.",
    },
    "leaderboard.hero.stat.updated": {
        text: "Updated",
        description: "Label under how long ago the leaderboard was last recomputed. Fits a narrow tile, so keep it short.",
    },
    "leaderboard.hero.updated.moments": {
        text: "moments ago",
        description: "How long ago the leaderboard was recomputed, when that was less than a minute ago.",
    },
    "leaderboard.hero.updated.minutes": {
        text: "{count}m ago",
        description: "How long ago the leaderboard was recomputed, in minutes. 'm' abbreviates minutes; the tile is narrow.",
    },
    "leaderboard.hero.updated.hours": {
        text: "{count}h ago",
        description: "How long ago the leaderboard was recomputed, in hours. 'h' abbreviates hours; the tile is narrow.",
    },
    "leaderboard.hero.updated.days": {
        text: "{count}d ago",
        description: "How long ago the leaderboard was recomputed, in days. 'd' abbreviates days; the tile is narrow.",
    },
    "leaderboard.hero.notice.lead": {
        text: "Heads up",
        description: "Bold opening of the development-status notice under the leaderboard heading.",
    },
    "leaderboard.hero.notice": {
        text: "{lead} - the leaderboard is in active development and the ranking formula is subject to change.",
        description: "Development-status notice under the leaderboard heading. {lead} is the bold opening labelled by leaderboard.hero.notice.lead and may move wherever the sentence needs it.",
    },
    "leaderboard.hero.blurb.item": {
        text: "Every player with a public profile, ranked by how much {item} they held when they last synced.",
        description: "One-sentence explanation under the title while the table is ranked by an item. {item} is the item's name, e.g. 'Originite Prime'.",
    },
    "leaderboard.hero.stat.holders": {
        text: "Holders",
        description: "Label under the count of players holding at least one of the ranked item.",
    },
    "leaderboard.hero.stat.topHolding": {
        text: "Top holding",
        description: "Label under the largest single quantity of the ranked item anyone holds.",
    },
    "leaderboard.hero.stat.asOf": {
        text: "Counted at",
        description: "Label of the freshness stat while ranked by an item; the value says the numbers are from each player's own last sync.",
    },
    "leaderboard.hero.stat.asOf.value": {
        text: "last sync",
        description: "Value of the freshness stat while ranked by an item: item counts are read at each player's own last sync, not on a schedule.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
