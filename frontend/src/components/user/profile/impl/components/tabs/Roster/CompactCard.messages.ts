import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "user";

export const messages = {
    "profile.roster.compact.levelAbbr": {
        text: "LV",
        description: "Two-letter caption over the level inside the compact card's level badge. Rendered uppercase; the badge is a small circle, so at most two or three characters fit.",
    },
    "profile.roster.compact.skillMastery": {
        text: "Skill {n}: M{mastery}",
        description: "Native tooltip on a skill slot of the compact card. {n} counts from 1; 'M3' is the game's shorthand for Mastery 3.",
    },
    "profile.roster.compact.skillLevel": {
        text: "Skill {n}: Lv.{level}",
        description: "Native tooltip on a skill slot with no mastery yet. 'Lv.' is the game's abbreviation for level.",
    },
    "profile.roster.compact.moduleStage": {
        text: "{module} Stage {level}",
        description: "Native tooltip on a module badge. {module} is the module's type designator from the game data and 'Stage' is the game's word for a module level.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
