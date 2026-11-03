import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "gacha";

export const messages = {
    "community.error.load": {
        text: "The community stats couldn't be loaded.",
        description: "Shown in place of the community page's stat panels (totals, most-pulled operators, rarity rates, pull timing) when they fail to load.",
    },
    "community.error.banners": {
        text: "The banner list couldn't be loaded.",
        description: "Shown above the banner runs panel on the community gacha page when the list of banners fails to load.",
    },
    "community.error.bannerStats": {
        text: "The per-banner pull counts couldn't be loaded, so the banners below show without them.",
        description: "Shown above the banner runs panel on the community gacha page when the per-banner pull statistics fail to load; the banners still list.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
