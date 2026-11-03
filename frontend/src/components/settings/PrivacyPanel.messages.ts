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
        text: "Who can see your roster and account stats.",
        description: "Card description over the visibility switches.",
    },
    "privacy.publicProfile.title": {
        text: "Public profile",
        description: "Label of the switch that makes the profile world-readable, and the switch's accessible name.",
    },
    "privacy.publicProfile.desc": {
        text: "Lets anyone with your UID open your profile, see your roster, and look up your scores.",
        description: "Caption under the public-profile switch.",
    },
    "privacy.leaderboards.title": {
        text: "Show on leaderboards",
        description: "Label of the switch that opts into public rankings, and the switch's accessible name.",
    },
    "privacy.leaderboards.desc": {
        text: "Lists you on {path}. Your score is calculated either way.",
        description: "Caption under the leaderboard switch. {path} is a URL path in monospace; move it wherever the sentence needs it.",
    },
    "privacy.leaderboards.path": {
        text: "/user/leaderboard",
        description: "URL path substituted into privacy.leaderboards.desc as {path}, shown in monospace. Leave it exactly as it is.",
    },
    "privacy.gacha.title": {
        text: "Store gacha history",
        description: "Label of the switch that keeps a record of the visitor's pulls, and the switch's accessible name. 'Gacha' is the game's random-draw system.",
    },
    "privacy.gacha.desc": {
        text: "Saves your synced pulls so you can browse them in Gacha → History and contribute aggregate stats.",
        description: "Caption under the gacha switch. 'Gacha → History' names a menu path in this site's own navigation.",
    },
    "privacy.publicGacha.title": {
        text: "Public pull history",
        description: "Label of the switch that lets visitors see the player's pull history on their profile and through links they share, and the switch's accessible name. 'Pull' is one draw on a headhunting banner.",
    },
    "privacy.publicGacha.desc": {
        text: "Shows your stored pulls on your profile's Pulls tab and through the links you share from Gacha → History. Off by default, and only on a public profile.",
        description: "Caption under the public pull history switch. 'Gacha → History' names a menu path in this site's own navigation.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
