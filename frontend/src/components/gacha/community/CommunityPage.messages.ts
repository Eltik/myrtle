import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "gacha";

export const messages = {
    "community.error.load": {
        text: "Couldn't load the community stats.",
        description: "Shown in place of the community page's stat panels (totals, most-pulled operators, rarity rates, pull timing) when they fail to load.",
    },
    "community.error.banners": {
        text: "Couldn't load the banner list.",
        description: "Shown above the banner runs panel on the community gacha page when the list of banners fails to load.",
    },
    "community.error.bannerStats": {
        text: "Couldn't load the per-banner pull counts, so the banners below show without them.",
        description: "Shown above the banner runs panel on the community gacha page when the per-banner pull statistics fail to load; the banners still list.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
