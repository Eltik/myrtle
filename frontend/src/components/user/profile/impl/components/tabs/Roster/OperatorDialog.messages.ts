import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

/**
 * The voice-language names are stored in a switch keyed by the API's language
 * codes, so that helper takes a `t` and returns resolved text.
 */
export const namespace = "user";

export const messages = {
    "profile.roster.dialog.elite": {
        text: "Elite",
        description: "Caption under the promotion badge in the operator dialog. 'Elite' is the game's promotion tier. Rendered uppercase by CSS.",
    },
    "profile.roster.dialog.combatStats": {
        text: "Combat Stats",
        description: "Section heading over the operator's battle attributes. Rendered uppercase by CSS.",
    },
    "profile.roster.dialog.skillLevelBadge": {
        text: "Lv. {level}",
        description: "Badge beside the Skills heading giving the shared skill level. 'Lv.' is the game's abbreviation.",
    },
    "profile.roster.dialog.default": {
        text: "Default",
        description: "Tag on the skill the operator has selected as its default. Rendered uppercase by CSS.",
    },
    "profile.roster.dialog.equipped": {
        text: "Equipped",
        description: "Tag on the module the operator currently has equipped. Rendered uppercase by CSS.",
    },
    "profile.roster.dialog.loading": {
        text: "Loading…",
        description: "Placeholder while the operator's game data is still being fetched. Ends with an ellipsis character.",
    },
    "profile.roster.dialog.info": {
        text: "Info",
        description: "Section heading over the recruitment date and voice language. Rendered uppercase by CSS.",
    },
    "profile.roster.dialog.recruited": {
        text: "Recruited",
        description: "Row label: when the account obtained this operator.",
    },
    "profile.roster.dialog.recruited.unknown": {
        text: "Unknown",
        description: "Stands in for the recruitment date when the game data does not report one.",
    },
    "profile.roster.dialog.voice": {
        text: "Voice",
        description: "Row label: which voice-over language the operator is set to.",
    },
    "profile.roster.voice.jp": {
        text: "Japanese",
        description: "Voice-over language name.",
    },
    "profile.roster.voice.cn": {
        text: "Chinese",
        description: "Voice-over language name: Mandarin Chinese.",
    },
    "profile.roster.voice.en": {
        text: "English",
        description: "Voice-over language name.",
    },
    "profile.roster.voice.kr": {
        text: "Korean",
        description: "Voice-over language name.",
    },
    "profile.roster.voice.cnTopolect": {
        text: "CN Regional",
        description: "Voice-over language name: a regional Chinese dialect. 'CN' is the country code for China; little room.",
    },
    "profile.roster.voice.linkage": {
        text: "Collab",
        description: "Voice-over option used for crossover operators, whose voice comes from the other franchise.",
    },
    "profile.roster.voice.ita": {
        text: "Italian",
        description: "Voice-over language name.",
    },
    "profile.roster.voice.rus": {
        text: "Russian",
        description: "Voice-over language name.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
