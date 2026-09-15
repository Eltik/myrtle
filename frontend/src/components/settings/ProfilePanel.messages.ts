import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "settings";

export const messages = {
    "resync.pending": {
        text: "Re-syncing…",
        description: "Label of the re-sync button while a sync is running. Shared by the Profile and Account & data panels. Ends in a single ellipsis character.",
    },
    "resync.now": {
        text: "Re-sync now",
        description: "Label of the button that pulls fresh game data. Shared by the Profile and Account & data panels.",
    },
    "profile.uid": {
        text: "UID {uid}",
        description: "Badge showing the visitor's in-game account number. UID is the game's own abbreviation; {uid} is the number, unformatted.",
    },
    "profile.synced": {
        text: "Synced {when}",
        description: "Badge saying how long ago the game data was pulled. {when} is an already-formatted short relative time such as '3h' or '2d'.",
    },
    "profile.gameSynced.title": {
        text: "Game-synced info",
        description: "Card title over the read-only fields that come from the game.",
    },
    "profile.gameSynced.desc": {
        text: "Read-only. Pulled from Yostar when you sync - change it in-game.",
        description: "Card description explaining that these fields can only be changed inside Arknights itself.",
    },
    "profile.nickname.title": {
        text: "Arknights nickname",
        description: "Field label for the in-game display name.",
    },
    "profile.nickname.desc": {
        text: "Shown on your profile, leaderboard, and tier lists you publish.",
        description: "Caption under the nickname field.",
    },
    "profile.level.title": {
        text: "Account level",
        description: "Field label for the in-game account level.",
    },
    "profile.level.desc": {
        text: "Doctor level from the in-game profile.",
        description: "Caption under the account-level field. 'Doctor' is what the game calls the player.",
    },
    "profile.level.value": {
        text: "Lv. {level}",
        description: "The account level as shown in the read-only field. 'Lv.' abbreviates level; {level} is the number, unformatted.",
    },
    "profile.server.title": {
        text: "Game server",
        description: "Field label for which regional game server the account lives on.",
    },
    "profile.server.desc": {
        text: "Which Arknights server you're synced from. Re-link your account to change this.",
        description: "Caption under the game-server field.",
    },
    "profile.assistant.title": {
        text: "Assistant operator",
        description: "Field label for the operator shown on the in-game home screen. 'Assistant' is the game's own term.",
    },
    "profile.assistant.desc": {
        text: "The operator displayed on your in-game and Myrtle profile.",
        description: "Caption under the assistant-operator field. 'Myrtle' is this site's name.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
