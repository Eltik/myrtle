import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "user";

export const messages = {
    "profile.stats.top.title": {
        text: "Top Operators",
        description: "Heading of the card showing the account's best-invested operators.",
    },
    "profile.stats.top.level": {
        text: "Lv {level}",
        description: "An operator's level on its tile. 'Lv' is the game's own abbreviation.",
    },
    "profile.stats.top.completeAria": {
        text: "{pct}% complete",
        description: "Accessible name of an operator's investment-progress bar.",
    },
    "profile.stats.top.fullyBuilt": {
        text: "Fully built",
        description: "Shown in an operator's hover card when nothing is left to invest in.",
    },
    "profile.stats.top.remaining": {
        text: "Remaining",
        description: "Heading over the list of upgrades an operator still needs. Rendered uppercase by CSS.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
