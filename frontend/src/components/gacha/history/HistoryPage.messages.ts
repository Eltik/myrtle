import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "gacha";

export const messages = {
    "history.signIn.title": {
        text: "Sign in to view your history",
        description: "Heading of the panel shown to a signed-out visitor on the pull-history page.",
    },
    "history.signIn.body": {
        text: "Your pull history, pity counters, and operator statistics are only available after linking your Yostar account.",
        description: "Paragraph telling a signed-out visitor why the page is empty. Yostar is the game's publisher and the account provider; 'pity' is the game's rising-chance mechanic.",
    },
    "history.signIn.action": {
        text: "Sign in",
        description: "Button that opens the sign-in dialog.",
    },
    "history.noData.title": {
        text: "No pull records found",
        description: "Heading of the panel shown when the signed-in player has no synced pulls yet.",
    },
    "history.noData.body": {
        text: "Sync your gacha records from the settings page to see your history, pity, and statistics here.",
        description: "Paragraph telling the player how to get their pull records onto this page. 'Pity' is the game's rising-chance mechanic.",
    },
    "history.noData.action": {
        text: "Go to settings",
        description: "Link to the settings page, where pull records are synced.",
    },
    "history.refresh": {
        text: "Refresh records",
        description: "Button that re-fetches the player's pull records from the game's servers.",
    },
    "history.refreshing": {
        text: "Refreshing…",
        description: "The refresh button's label while the fetch is in flight. Keep the single-character ellipsis.",
    },
    "history.refresh.okTitle": {
        text: "Up to date",
        description: "Toast title after the pull records were re-fetched successfully.",
    },
    "history.refresh.okBody": {
        text: "Synced {count} records from Yostar.",
        description: "Toast body after a successful refresh, saying how many pull records came back. Yostar is the game's publisher.",
    },
    "history.refresh.errTitle": {
        text: "Couldn't refresh",
        description: "Toast title when re-fetching the pull records failed.",
    },
    "history.refresh.errBody": {
        text: "Try signing in again.",
        description: "Toast body suggesting a fix when the refresh failed and the server sent no message of its own.",
    },
    "history.error.load": {
        text: "Couldn’t load pull records.",
        description: "Bold lead-in of the error banner on the pull-history page; the server's own message follows it on the same line.",
    },
    "history.error.unknown": {
        text: "Unknown error.",
        description: "Stand-in for the server's error message when the failed request carried none.",
    },
    "history.header.kicker": {
        text: "Personal · pull history",
        description: "Small uppercase label above the pull-history heading. Keep the middle dot.",
    },
    "history.header.title": {
        text: "Your {emphasis} history.",
        description: "Heading of the pull-history page. {emphasis} is the accent-coloured word labelled by history.header.titleEmphasis and may sit anywhere the sentence needs it. Keep the full stop.",
    },
    "history.header.titleEmphasis": {
        text: "gacha",
        description: "The accent-coloured word substituted into history.header.title as {emphasis}. 'Gacha' is the usual word for the game's paid randomised recruitment.",
    },
    "history.header.blurb": {
        text: "Pull counts, current pity, your most-pulled operators, and a full history across every banner type.",
        description: "Paragraph under the pull-history heading. 'Pity' is the game's rising-chance mechanic.",
    },
    "history.header.communityLink": {
        text: "View community stats →",
        description: "Link from the personal history page to the community-wide figures. Keep the arrow.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
