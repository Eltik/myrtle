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
    "profile.level.value": {
        text: "Lv. {level}",
        description: "The account level as shown in the read-only field. 'Lv.' abbreviates level; {level} is the number, unformatted.",
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
