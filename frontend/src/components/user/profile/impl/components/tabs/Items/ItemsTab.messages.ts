import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

/**
 * The sort and rarity options live in tables, because each select's trigger
 * renders the chosen option through its table rather than from the item it was
 * picked from.
 */
export const namespace = "user";

export const messages = {
    "profile.items.aria": {
        text: "Inventory items",
        description: "Accessible name of the inventory section of a profile.",
    },
    "profile.items.search.placeholder": {
        text: "Search items...",
        description: "Prompt inside the empty inventory search box. Keep the three dots as written.",
    },
    "profile.items.sort.placeholder": {
        text: "Sort by",
        description: "Shown in the sort select before anything is chosen.",
    },
    "profile.items.sort.rarity": {
        text: "Sort by Rarity",
        description: "Sort option: by the item's star rating.",
    },
    "profile.items.sort.qty": {
        text: "Sort by Quantity",
        description: "Sort option: by how many the account holds.",
    },
    "profile.items.sort.name": {
        text: "Sort by Name",
        description: "Sort option: alphabetically.",
    },
    "profile.items.sort.category": {
        text: "Sort by Category",
        description: "Sort option: grouped by inventory category.",
    },
    "profile.items.sort.asc": {
        text: "Asc",
        description: "Label on the sort-direction button while the order is smallest-first. Abbreviated from 'ascending'; little room.",
    },
    "profile.items.sort.desc": {
        text: "Desc",
        description: "Label on the sort-direction button while the order is largest-first. Abbreviated from 'descending'; little room.",
    },
    "profile.items.rarity.placeholder": {
        text: "Filter by Rarity",
        description: "Shown in the rarity select before anything is chosen.",
    },
    "profile.items.rarity.all": {
        text: "All Rarities",
        description: "Rarity filter: every star rating.",
    },
    "profile.items.rarity.5": {
        text: "5 Star",
        description: "Rarity filter: five-star items. Always written in this one form.",
    },
    "profile.items.rarity.4": {
        text: "4 Star",
        description: "Rarity filter: four-star items. Always written in this one form.",
    },
    "profile.items.rarity.3": {
        text: "3 Star",
        description: "Rarity filter: three-star items. Always written in this one form.",
    },
    "profile.items.rarity.2": {
        text: "2 Star",
        description: "Rarity filter: two-star items. Always written in this one form.",
    },
    "profile.items.rarity.1": {
        text: "1 Star",
        description: "Rarity filter: one-star items. Always written in this one form.",
    },
    "profile.items.viewMode.aria": {
        text: "View mode",
        description: "Accessible name of the pair of buttons that switch between card layouts.",
    },
    "profile.items.viewMode.detailed": {
        text: "Detailed view",
        description: "Accessible name of the button for the large item cards.",
    },
    "profile.items.viewMode.compact": {
        text: "Compact view",
        description: "Accessible name of the button for the small item cards.",
    },
    "profile.items.count.items": {
        text: "items",
        description: "Follows the number of item kinds shown, in the toolbar summary. Always in this one form; rendered uppercase by CSS.",
    },
    "profile.items.count.total": {
        text: "total",
        description: "Follows the summed quantity of every item shown, in the toolbar summary. Rendered uppercase by CSS.",
    },
    "profile.items.empty.kicker": {
        text: "Inventory",
        description: "Kicker over the inventory empty state. Rendered uppercase by CSS.",
    },
    "profile.items.empty.filtered.title": {
        text: "No items match",
        description: "Empty-state title when the filters leave nothing.",
    },
    "profile.items.empty.filtered.desc": {
        text: "Try clearing filters or a different category.",
        description: "Empty-state body when the filters leave nothing.",
    },
    "profile.items.empty.none.title": {
        text: "No items yet",
        description: "Empty-state title when the account has no inventory on file.",
    },
    "profile.items.empty.none.desc": {
        text: "This Doctor's inventory is empty.",
        description: "Empty-state body when the account has no inventory on file. 'Doctor' is what Arknights calls the player; keep the apostrophe.",
    },
} satisfies MessageMap;

// `dynamic`: the sort and rarity names are resolved through the SORT_LABELS and
// RARITY_OPTIONS tables, which the extractor cannot see.
export const { keys } = defineMessages({ namespace, messages, dynamic: true });
