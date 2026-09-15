import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

/**
 * The inventory's own taxonomy, declared next to the tables that hold it: the
 * category tables in `helpers.ts` are plain constants, so they carry message
 * KEYS and whichever card renders one resolves it with `t()`.
 *
 * These are this site's groupings, not the game's item names - the names come
 * from the material data. `formatItemType` and `formatVoucherId` are left out
 * on purpose: both mechanically title-case a raw API code.
 */
export const namespace = "user";

export const messages = {
    "profile.items.openAria": {
        text: "Open details for {name}",
        description: "Accessible name of an inventory card, which opens that item's dialog. {name} comes from the game data.",
    },
    "profile.items.category.all": {
        text: "All",
        description: "Category chip: every item in the inventory.",
    },
    "profile.items.category.exp": {
        text: "EXP",
        description: "Category chip: the experience items. 'EXP' is the game's own abbreviation.",
    },
    "profile.items.category.lmd": {
        text: "Currency",
        description: "Category chip: the in-game currencies.",
    },
    "profile.items.category.mat": {
        text: "Materials",
        description: "Category chip: the crafting materials.",
    },
    "profile.items.category.skill": {
        text: "Skill Books",
        description: "Category chip: the items that raise skill levels.",
    },
    "profile.items.category.module": {
        text: "Module Mats",
        description: "Category chip: the materials for module upgrades. 'Mats' is short for materials; the chip is narrow.",
    },
    "profile.items.category.chip": {
        text: "Chips",
        description: "Category chip: the promotion chips, which are class-specific.",
    },
    "profile.items.category.furniture": {
        text: "Furniture",
        description: "Category chip: the base furniture.",
    },
    "profile.items.category.ticket": {
        text: "Tickets",
        description: "Category chip: vouchers and tickets.",
    },
    "profile.items.category.consume": {
        text: "Consumables",
        description: "Category chip: single-use items.",
    },
    "profile.items.category.other": {
        text: "Other",
        description: "Category chip: everything that fits no other category.",
    },
    "profile.items.itemLabel.all": {
        text: "Item",
        description: "What one entry of this category is called, shown on its card. The generic fallback.",
    },
    "profile.items.itemLabel.exp": {
        text: "Battle Record",
        description: "What one experience item is called. 'Battle Record' is the game's own name for them.",
    },
    "profile.items.itemLabel.lmd": {
        text: "Currency",
        description: "What one currency entry is called on its card.",
    },
    "profile.items.itemLabel.mat": {
        text: "Material",
        description: "What one crafting material is called on its card.",
    },
    "profile.items.itemLabel.skill": {
        text: "Skill book",
        description: "What one skill-level item is called on its card.",
    },
    "profile.items.itemLabel.module": {
        text: "Module mat",
        description: "What one module material is called on its card. 'Mat' is short for material; little room.",
    },
    "profile.items.itemLabel.chip": {
        text: "Chip",
        description: "What one promotion chip is called on its card.",
    },
    "profile.items.itemLabel.furniture": {
        text: "Furniture",
        description: "What one furniture item is called on its card.",
    },
    "profile.items.itemLabel.ticket": {
        text: "Ticket",
        description: "What one ticket or voucher is called on its card.",
    },
    "profile.items.itemLabel.consume": {
        text: "Consumable",
        description: "What one single-use item is called on its card.",
    },
    "profile.items.itemLabel.other": {
        text: "Item",
        description: "What one uncategorized entry is called on its card.",
    },
} satisfies MessageMap;

// `dynamic`: the category keys are stored in the CATEGORY_LABELS and
// CATEGORY_ITEM_LABELS tables and resolved as `t(CATEGORY_LABELS[c])`, so the
// extractor has no literal call site to match them against.
export const { keys } = defineMessages({ namespace, messages, dynamic: true });
