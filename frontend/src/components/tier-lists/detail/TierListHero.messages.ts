import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tierLists";

export const messages = {
    "detail.hero.breadcrumb": {
        text: "Tier Lists",
        description: "Breadcrumb link back to the tier-list browse page.",
    },
    "detail.hero.official": {
        text: "Official",
        description: "Badge marking a tier list maintained by the site's own team. Rendered uppercase.",
    },
    "detail.hero.community": {
        text: "Community",
        description: "Badge marking a tier list built by a player. Rendered uppercase.",
    },
    "detail.hero.trending": {
        text: "Trending",
        description: "Badge marking a list with unusual recent traffic. Rendered uppercase.",
    },
    "detail.hero.kicker": {
        text: "Tier List",
        description: "Small uppercase label above the list's title.",
    },
    "detail.hero.authorFallback": {
        text: "Player",
        description: "Stand-in for an author who has set no nickname. 'Player' is the site's term; the game itself says 'Doctor'.",
    },
    "detail.hero.authorUnknown": {
        text: "Unknown author",
        description: "Shown in place of the author when the list has none on record.",
    },
    "detail.hero.updated": {
        text: "Updated {when}",
        description: "How long ago the list was last edited, e.g. 'Updated 3 days ago'.",
    },
    "detail.hero.views": {
        text: "views",
        description: "Unit after the list's view count.",
    },
    "detail.hero.views.title": {
        text: "{count} views",
        description: "Tooltip on the view count, with the exact number.",
    },
    "detail.hero.favorites": {
        text: "favorites",
        description: "Unit after the count of visitors who saved the list.",
    },
    "detail.hero.favorites.title": {
        text: "{count} favorites",
        description: "Tooltip on the favourite count, with the exact number.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
