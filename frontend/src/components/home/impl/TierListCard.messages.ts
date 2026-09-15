import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "home";

export const messages = {
    "card.trending": {
        text: "· trending",
        description: "Suffix appended to a tier-list card's tag pill when the list is trending. The leading middle dot is a separator and part of the label.",
    },
    "card.picks": {
        text: "{count, plural, one {# pick} other {# picks}}",
        description: "How many operators sit in the card's top tier. Shown to the right of the tier name.",
    },
    "card.ghost": {
        text: "+ {names}{extra} {count, plural, one {tier} other {tiers}}",
        description: "Summary of the tiers below the top one on a tier-list card, e.g. '+ B, C, D +2 tiers'. {names} is a comma-joined list of tier names from the data, {extra} is an already-formatted ' +N' overflow marker or empty, and {count} is the total number of remaining tiers.",
    },
    "card.opTitle": {
        text: "{name} · {role}",
        description: "Hover title of an operator chip on a tier-list card. Both values come from the game data and are not translated.",
    },
    "card.views": {
        text: "{count} views",
        description: "Hover title of the view counter on a tier-list card. Always plural in the source, and {count} is rendered unformatted to match the number shown next to it.",
    },
    "card.favorites": {
        text: "{count} favorites",
        description: "Hover title of the favourite counter on a tier-list card. Always plural in the source, and {count} is rendered unformatted.",
    },
    "card.viewsIcon": {
        text: "Views",
        description: "Accessible name of the eye icon in front of a tier-list card's view count.",
    },
    "card.votesIcon": {
        text: "Votes",
        description: "Accessible name of the chevron icon in front of a tier-list card's favourite count.",
    },
    "card.open": {
        text: "Open",
        description: "Call to action in the footer of a tier-list card. A verb: it opens the full list.",
    },
    "card.arrowIcon": {
        text: "Right arrow",
        description: "Accessible name of the decorative arrow icon after the 'Open' call to action.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
