import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "settings";

export const messages = {
    "privacy.alert.title": {
        text: "What's public, and what isn't",
        description: "Title of the info banner at the top of the Privacy panel.",
    },
    "privacy.alert.body": {
        text: "Public profiles show your nickname, UID, roster, account scores, and leaderboard rankings. Authentication tokens, settings, saved DPS configs, and email are always private.",
        description: "Body of the info banner. UID is the in-game account number; 'roster' is the set of owned operators; DPS is damage per second.",
    },
    "privacy.visibility.title": {
        text: "Profile visibility",
        description: "Card title over the visibility switches.",
    },
    "privacy.visibility.desc": {
        text: "Who can see your roster and account stats. Backed by {column}.",
        description: "Card description over the visibility switches. {column} is a database column name in monospace; move it wherever the sentence needs it.",
    },
    "privacy.visibility.column": {
        text: "user_settings.public_profile",
        description: "Database column name substituted into privacy.visibility.desc as {column}, shown in monospace. An identifier, not prose - leave it exactly as it is.",
    },
    "privacy.publicProfile.title": {
        text: "Public profile",
        description: "Label of the switch that makes the profile world-readable.",
    },
    "privacy.publicProfile.desc": {
        text: "Lets anyone with your UID open your profile, see your roster, and look up your scores.",
        description: "Caption under the public-profile switch.",
    },
    "privacy.leaderboards.title": {
        text: "Show on leaderboards",
        description: "Label of the switch that opts into public rankings.",
    },
    "privacy.leaderboards.desc": {
        text: "Opt in to ranked appearance on {path}. Scores are still calculated either way. Backed by {column}.",
        description: "Caption under the leaderboard switch. {path} is a URL path and {column} a database column name, both in monospace; move either wherever the sentence needs it.",
    },
    "privacy.leaderboards.path": {
        text: "/user/leaderboard",
        description: "URL path substituted into privacy.leaderboards.desc as {path}, shown in monospace. Leave it exactly as it is.",
    },
    "privacy.leaderboards.column": {
        text: "share_stats",
        description: "Database column name substituted into privacy.leaderboards.desc as {column}, shown in monospace. Leave it exactly as it is.",
    },
    "privacy.gacha.title": {
        text: "Store gacha history",
        description: "Label of the switch that keeps a record of the visitor's pulls. 'Gacha' is the game's random-draw system.",
    },
    "privacy.gacha.desc": {
        text: "Saves your synced pulls so you can browse them in Gacha → History and contribute aggregate stats. Backed by {column}.",
        description: "Caption under the gacha switch. 'Gacha → History' names a menu path in this site's own navigation; {column} is a database column name in monospace, movable wherever the sentence needs it.",
    },
    "privacy.gacha.column": {
        text: "store_gacha",
        description: "Database column name substituted into privacy.gacha.desc as {column}, shown in monospace. Leave it exactly as it is.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
