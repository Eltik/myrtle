import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "user";

export const messages = {
    "leaderboard.you.title": {
        text: "Your standing",
        description: "Heading of the card showing the signed-in player's own leaderboard position.",
    },
    "leaderboard.you.rank": {
        text: "Rank",
        description: "Sits directly before the player's own rank number, which follows in bold after a space.",
    },
    "leaderboard.you.topPercentile": {
        text: " · Top ",
        description: "Separator and lead-in before the player's percentile, e.g. ' · Top 4.2%'. Keep the spaces on both sides; the percentage follows in bold.",
    },
    "leaderboard.you.metric.grade": {
        text: "Grade",
        description: "Label of the small tile holding the player's letter grade. Very narrow tile.",
    },
    "leaderboard.you.metric.score": {
        text: "Score",
        description: "Label of the small tile holding the player's total score. Very narrow tile.",
    },
    "leaderboard.you.viewProfile": {
        text: "View profile",
        description: "Button that opens the signed-in player's own profile page.",
    },
    "leaderboard.you.share": {
        text: "Share profile",
        description: "Accessible name of the icon button that copies a link to the player's profile.",
    },
    "leaderboard.you.share.copied.title": {
        text: "Profile link copied",
        description: "Toast title after the profile link is copied to the clipboard.",
    },
    "leaderboard.you.share.copied.desc": {
        text: "Your profile link is on the clipboard.",
        description: "Toast body after the profile link is copied to the clipboard.",
    },
    "leaderboard.you.share.failed.title": {
        text: "Couldn't copy",
        description: "Toast title when copying the profile link failed. Keep the apostrophe.",
    },
    "leaderboard.you.share.failed.desc": {
        text: "Clipboard access was denied.",
        description: "Toast body when the browser refused clipboard access.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
