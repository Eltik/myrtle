import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "user";

export const messages = {
    "profile.plans.kicker": {
        text: "Plans",
        description: "Kicker over the empty state of the profile's plans tab. Rendered uppercase by CSS.",
    },
    "profile.plans.empty.title": {
        text: "No public plans",
        description: "Empty-state title when the account has pinned no plans to its profile.",
    },
    "profile.plans.empty.desc": {
        text: "This player hasn't pinned any plans to their profile yet.",
        description: "Empty-state body of the plans tab. 'Doctor' is what Arknights calls the player; keep the apostrophe.",
    },
    "profile.plans.aria": {
        text: "Operator plans",
        description: "Accessible name of the plans section of a profile.",
    },
    "profile.plans.level": {
        text: "Level",
        description: "Row label over an operator's current and target level.",
    },
    "profile.plans.skills": {
        text: "Skills",
        description: "Row label over an operator's current and target skill ranks.",
    },
    "profile.plans.modules": {
        text: "Modules",
        description: "Row label over an operator's current and target module levels.",
    },
    "profile.plans.skillLevel": {
        text: "Lv.{level}",
        description: "An operator's level on a plan card. 'Lv.' is the game's abbreviation; very little room.",
    },
    "profile.plans.eliteAlt": {
        text: "Elite {elite}",
        description: "Alt text of the promotion badge on a plan card. 'Elite' is the game's promotion tier.",
    },
    "profile.plans.skillFallback": {
        text: "Skill {n}",
        description: "Stand-in name for a skill the game data does not name; {n} counts from 1.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
