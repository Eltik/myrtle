import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

/**
 * The shop-category table in `plan.ts` is plain data in a module with no
 * React, so it carries message KEYS and `PlannerTab` resolves them with `t()`.
 * The store-sale row's English name is derived there too, so `usePlanData`
 * passes its own `t` in.
 *
 * Item names come from the game data and are never translated here.
 */
export const namespace = "tools";

export const messages = {
    "release.plan.storeSale": {
        text: "Store sale",
        description: "Name of the synthetic row grouping outfits that go on sale in the store without a matching event.",
    },
    "release.shopKind.outfit": {
        text: "Outfits",
        description: "Event-shop category: operator outfits. 'Outfit' is the game's own word for a skin.",
    },
    "release.shopKind.furniture": {
        text: "Furniture",
        description: "Event-shop category: dormitory furniture.",
    },
    "release.shopKind.material": {
        text: "Materials",
        description: "Event-shop category: crafting materials.",
    },
    "release.shopKind.currency": {
        text: "LMD and supplies",
        description: "Event-shop category: money and consumables. 'LMD' is the game's own currency and stays as-is.",
    },
    "release.shopKind.exp": {
        text: "EXP cards",
        description: "Event-shop category: experience cards. 'EXP' is the game's own abbreviation.",
    },
    "release.shopKind.ticket": {
        text: "Tickets",
        description: "Event-shop category: recruitment and headhunting tickets.",
    },
    "release.shopKind.other": {
        text: "Other",
        description: "Event-shop category for everything that fits no other category.",
    },
} satisfies MessageMap;

// `dynamic`: the shop-category keys are stored in a lookup table and resolved
// by the consuming component as `t(SHOP_KIND_LABEL_KEYS[kind])`, so the
// extractor has no literal call site to match them against.
export const { keys } = defineMessages({ namespace, messages, dynamic: true });
