import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "gacha";

export const messages = {
    "history.pity.kicker": {
        text: "Current pity",
        description: "Small uppercase label above the pity counters. 'Pity' is the game's mechanic where a 6-star becomes likelier the longer one goes without.",
    },
    "history.pity.title": {
        text: "Pulls since last 6★.",
        description: "Heading of the panel showing how many pulls each banner bucket has gone without a 6-star.",
    },
    "history.pity.cardLabel": {
        text: "{type} banner",
        description: "Label on a pity card naming its banner bucket, e.g. 'Limited banner'. Rendered uppercase.",
    },
    "history.pity.pullsUnit": {
        text: "pulls",
        description: "Unit printed small after the pity count on a card.",
    },
    "history.pity.total": {
        text: "{count} total",
        description: "On a pity card: how many pulls the player has made on that bucket in total.",
    },
    "history.pity.bannerEnded": {
        text: "banner ended",
        description: "Pill on a pity card whose counter was reset because the banner it belonged to has closed. Rendered uppercase.",
    },
    "history.pity.softPity": {
        text: "soft pity",
        description: "Pill on a pity card past the point where the 6-star chance starts climbing. Rendered uppercase.",
    },
    "history.pity.nearGuaranteed": {
        text: "near guaranteed",
        description: "Pill on a pity card within five pulls of the guaranteed 6-star. Rendered uppercase.",
    },
    "history.pity.softPityAt.title": {
        text: "Soft pity at {count}",
        description: "Tooltip on the tick marking where the 6-star chance starts climbing.",
    },
    "history.pity.softPityAt": {
        text: "soft pity at {count}",
        description: "Under a pity bar, at its left: the pull count where the 6-star chance starts climbing. Lowercase.",
    },
    "history.pity.guaranteedAt": {
        text: "guaranteed at {count}",
        description: "Under a pity bar, at its right: the pull count at which a 6-star is certain. Lowercase.",
    },
    "history.pity.resetNote": {
        text: "Pity does not carry between {type} banners. Your last pull was on “{banner},” which has ended.",
        description: "Explains a pity counter reset to zero. {banner} is the banner's name from the game data; keep the curly quotes around it.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
