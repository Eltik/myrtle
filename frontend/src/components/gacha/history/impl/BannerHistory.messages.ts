import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "gacha";

export const messages = {
    "history.banners.kicker": {
        text: "Pull history",
        description: "Small uppercase label above the full list of the player's pulls.",
    },
    "history.banners.title": {
        text: "Every pull, sorted newest first.",
        description: "Heading of the panel listing every pull the player has made.",
    },
    "history.banners.empty": {
        text: "No pulls recorded for this banner type.",
        description: "Empty state when the selected banner bucket holds none of the player's pulls.",
    },
    "history.banners.emptyNoRarity": {
        text: "Select at least one rarity to display pulls.",
        description: "Empty state when every rarity chip has been switched off, so nothing can be listed.",
    },
    "history.banners.emptyFiltered": {
        text: "No {rarities} pulls in {tab}. Other rarities are hidden by the filter.",
        description: "Empty state when the rarity filter leaves the selected banner bucket with nothing. {rarities} is a list of star counts like '6★ 5★'; {tab} is the bucket's name.",
    },
    "history.banners.showAllRarities": {
        text: "Show all rarities",
        description: "Button inside the empty state that switches every rarity chip back on.",
    },
    "history.banners.col.operator": {
        text: "Operator",
        description: "Column heading for the operator a pull produced. Rendered uppercase.",
    },
    "history.banners.col.banner": {
        text: "Banner",
        description: "Column heading for the banner a pull was made on. Rendered uppercase.",
    },
    "history.banners.col.date": {
        text: "Date",
        description: "Column heading for when a pull was made. Rendered uppercase.",
    },
    "history.banners.range": {
        text: "{from}-{to} of {total} pulls",
        description: "Pager summary under the table, e.g. '51-100 of 2,431 pulls'.",
    },
    "history.banners.prev": {
        text: "Prev",
        description: "Pager button for the previous page. Abbreviated; the button is tiny.",
    },
    "history.banners.next": {
        text: "Next",
        description: "Pager button for the next page. The button is tiny.",
    },
    "history.banners.rarityLabel": {
        text: "Rarity",
        description: "Label before the row of rarity filter chips. Rendered uppercase.",
    },
    "history.banners.rarityChip": {
        text: "{rarity} star ({count} pulls)",
        description: "Accessible name of one rarity filter chip, naming the rarity and how many of the player's pulls it holds.",
    },
    "history.banners.rarity.none": {
        text: "No rarities selected",
        description: "Describes the rarity filter when every chip is switched off. Read out to screen readers.",
    },
    "history.banners.rarity.only": {
        text: "{rarity}★ only",
        description: "Describes the rarity filter when exactly one chip is on, e.g. '6★ only'. Read out to screen readers.",
    },
    "history.banners.status.filtered": {
        text: "Filtered to {rarities}. Showing {shown} of {total} {tab} pulls.",
        description: "Screen-reader-only summary of the active filter. {rarities} describes the rarity chips; {tab} is the banner bucket's name.",
    },
    "history.banners.status.all": {
        text: "Showing all rarities.",
        description: "Screen-reader-only summary when no rarity is filtered out.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
