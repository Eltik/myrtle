import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

/**
 * The outfit browser. Skin, brand and operator names come from the game data;
 * only this dialog's own chrome lives here.
 *
 * The acquisition-channel chips and the price chips are produced by plain
 * functions in `SkinViewerDialog.tsx`, so those hold message KEYS and the
 * caller resolves them with `t()`.
 */
export const namespace = "user";

export const messages = {
    "profile.skins.title": {
        text: "Skin Collection",
        description: "Title of the outfit-browser dialog, also its screen-reader title.",
    },
    "profile.skins.header.missing": {
        text: "missing",
        description: "Follows the count of outfits not owned, in the dialog header. Lowercase; a middle dot separates it from the next figure.",
    },
    "profile.skins.header.owned": {
        text: "owned",
        description: "Follows the count of outfits owned, in the dialog header. Lowercase.",
    },
    "profile.skins.header.total": {
        text: "total",
        description: "Follows the total outfit count, in the dialog header. Lowercase.",
    },
    "profile.skins.filter.all": {
        text: "All",
        description: "Ownership filter: every outfit. Rendered uppercase by CSS, with a count beside it.",
    },
    "profile.skins.filter.owned": {
        text: "Owned",
        description: "Ownership filter: only outfits the account has. Rendered uppercase by CSS.",
    },
    "profile.skins.filter.missing": {
        text: "Missing",
        description: "Ownership filter: only outfits the account lacks. Rendered uppercase by CSS.",
    },
    "profile.skins.sort.label": {
        text: "Sort",
        description: "Caption in front of the sort buttons. Rendered uppercase by CSS.",
    },
    "profile.skins.sort.brand": {
        text: "Brand",
        description: "Sort option: group outfits by their in-game brand or series. Rendered uppercase by CSS.",
    },
    "profile.skins.sort.date": {
        text: "Date",
        description: "Sort option: newest outfits first. Rendered uppercase by CSS.",
    },
    "profile.skins.sort.popularity": {
        text: "Popularity",
        description: "Sort option: most widely owned outfits first. Rendered uppercase by CSS.",
    },
    "profile.skins.search.aria": {
        text: "Search skins",
        description: "Accessible name of the outfit search box.",
    },
    "profile.skins.search.placeholder": {
        text: "Search by operator or skin…",
        description: "Placeholder in the outfit search box. Ends with an ellipsis character.",
    },
    "profile.skins.search.clear": {
        text: "Clear search",
        description: "Accessible name of the button that empties the search box.",
    },
    "profile.skins.empty.title": {
        text: "No skins match your filters",
        description: "Empty state when the search and filters leave no outfits.",
    },
    "profile.skins.empty.withQuery": {
        text: "Try a different search or switch the ownership filter.",
        description: "Empty-state hint when a search term is active.",
    },
    "profile.skins.empty.noQuery": {
        text: "Try switching the ownership filter.",
        description: "Empty-state hint when no search term is active.",
    },
    "profile.skins.card.aria": {
        text: "View {operator} · {skin}",
        description: "Accessible name of an outfit card. Both names come from the game data; keep the middle dot.",
    },
    "profile.skins.card.fallbackName": {
        text: "Skin",
        description: "Stand-in name for an outfit the game data does not name.",
    },
    "profile.skins.badge.owned": {
        text: "Owned",
        description: "Accessible name of the check badge on an owned outfit's card.",
    },
    "profile.skins.badge.missing": {
        text: "Missing",
        description: "Accessible name of the empty badge on an outfit the account lacks.",
    },
    "profile.skins.popularity.tooltip": {
        text: "{owners} owners ({pct}% of imported users)",
        description: "Tooltip on an outfit's popularity chip. 'Imported users' are the accounts synced to this site.",
    },
    "profile.skins.detail.close": {
        text: "Back to collection",
        description: "Button that closes one outfit's detail view and returns to the browser.",
    },
    "profile.skins.detail.popularity": {
        text: "Popularity",
        description: "Row label in an outfit's detail view, over the share of accounts that own it.",
    },
    "profile.skins.detail.ofUsers": {
        text: "of users own this",
        description: "Follows the percentage in that row; the owner count follows after a middle dot.",
    },
    "profile.skins.detail.ownerCount": {
        text: "{n} owners",
        description: "Closes that row: how many synced accounts own the outfit.",
    },
    "profile.skins.section.all": {
        text: "All Skins",
        description: "Heading over the single ungrouped section, used when sorting by date or popularity.",
    },
    "profile.skins.section.other": {
        text: "Other",
        description: "Section heading for outfits whose brand the game data does not name.",
    },
    "profile.skins.channel.collab": {
        text: "Collab",
        description: "Chip on a brand section: outfits from a crossover with another franchise.",
    },
    "profile.skins.channel.seasonal": {
        text: "Seasonal",
        description: "Chip on a brand section: outfits from the game's seasonal attire line.",
    },
    "profile.skins.channel.specialPack": {
        text: "Special Pack",
        description: "Chip on a brand section: outfits sold inside a bundle.",
    },
    "profile.skins.channel.is": {
        text: "IS Reward",
        description: "Chip on a brand section: outfits earned in Integrated Strategies. 'IS' is the game mode's own abbreviation.",
    },
    "profile.skins.channel.event": {
        text: "Event Reward",
        description: "Chip on a brand section: outfits earned from a limited-time event.",
    },
    "profile.skins.channel.codeExchange": {
        text: "Code Exchange",
        description: "Chip on a brand section: outfits redeemed with a code.",
    },
    "profile.skins.price.is": {
        text: "IS",
        description: "Price chip on an outfit earned in Integrated Strategies. The game mode's own abbreviation, normally unchanged.",
    },
    "profile.skins.price.free": {
        text: "Free",
        description: "Price chip on an outfit that costs nothing. Very little room.",
    },
    "profile.skins.price.bundle": {
        text: "Bundle",
        description: "Price chip on an outfit sold only inside a pack. Very little room.",
    },
    "profile.skins.price.bundle.tooltip": {
        text: "Obtain from Special Pack",
        description: "Tooltip on the Bundle chip. 'Special Pack' is the in-game store bundle.",
    },
    "profile.skins.price.op": {
        text: "{op} OP",
        description: "Price chip: the cost in Originite Prime. 'OP' is the community abbreviation for that in-game currency.",
    },
    "profile.skins.price.store.tooltip": {
        text: "Outfit Store",
        description: "Tooltip on a paid outfit's price chip, naming the in-game store it is sold in.",
    },
} satisfies MessageMap;

// `dynamic`: the filter, sort, channel and price keys are stored in the tables
// and helpers of `SkinViewerDialog.tsx` and resolved as `t(tab.labelKey)`, so
// the extractor has no literal call site to match them against.
export const { keys } = defineMessages({ namespace, messages, dynamic: true });
