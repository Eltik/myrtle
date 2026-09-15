import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tierLists";

export const messages = {
    "detail.tier.label": {
        text: "Tier {name} - view details",
        description: "Accessible name of a tier's label button. {name} is the tier's own label, written by the list's author.",
    },
    "detail.tier.labelWithDescription": {
        text: "Tier {name} - {description} - view details",
        description: "Accessible name of a tier's label button when the author wrote a description for it. Both {name} and {description} are the author's own words.",
    },
    "detail.tier.empty": {
        text: "No operators in this tier yet.",
        description: "Shown in a tier row the author has placed no operators in.",
    },
    "detail.tier.openDetails": {
        text: "Open tier {name} details",
        description: "Accessible name of the hover card, which opens the tier's full details. {name} is the tier's own label.",
    },
    "detail.tier.kicker": {
        text: "Tier",
        description: "Small uppercase label above the tier's name in its hover card.",
    },
    "detail.tier.noDescription": {
        text: "No description provided for this tier.",
        description: "Stand-in in the hover card when the author wrote no description for the tier.",
    },
    "detail.tier.classCount": {
        text: "{count, plural, one {# {profession}} other {# {profession}s}}",
        description: "Tooltip on a class chip in the hover card, e.g. '3 Guards'. {profession} is the class name from the game data; English pluralises it by adding s, so use whatever form your language needs.",
    },
    "detail.tier.viewDetails": {
        text: "View tier details",
        description: "Footer of the tier hover card, saying the card opens the tier's full details. Rendered uppercase.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
