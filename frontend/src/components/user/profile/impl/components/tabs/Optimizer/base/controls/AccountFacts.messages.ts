import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "user";

export const messages = {
    "profile.base.facts.recruitSlots": {
        text: "Recruit slots",
        description: "Label in front of the recruit-slot buttons. 'Recruit slots' are the in-game hiring queue's slots.",
    },
    "profile.base.facts.recruitSlots.aria": {
        text: "Recruit slots purchased beyond the first",
        description: "Accessible name of those buttons.",
    },
    "profile.base.facts.recruitSlots.tooltip": {
        text: "Recruitment slots you have purchased beyond the first - account state the sync cannot read. Declaring it prices skills like Lin’s Meritocracy (+10% HR speed per slot); left at +0 they honestly count as zero.",
        description: "Tooltip over the account-facts row. 'Lin' is an operator and 'Meritocracy' her base skill - both keep the game's names; 'HR' is the base's hiring office. The apostrophe is a typographic one.",
    },
    "profile.base.facts.training": {
        text: "Training",
        description: "Label in front of the class-being-trained select. Refers to the base's Training Room.",
    },
    "profile.base.facts.training.aria": {
        text: "Class currently training",
        description: "Accessible name of that select.",
    },
    "profile.base.facts.training.nobody": {
        text: "Nobody",
        description: "Option in that select: no operator is being trained. The class names beside it come from the game data.",
    },
    "profile.base.facts.saved": {
        text: "saved",
        description: "Marker that the declared facts are stored on the account. Lowercase on purpose.",
    },
    "profile.base.facts.whatIf": {
        text: "what-if",
        description: "Marker that the declared facts are a temporary experiment, not stored. Lowercase on purpose.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
