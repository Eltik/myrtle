import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tierLists";

export const messages = {
    "my.sort.recent": {
        text: "Recently updated",
        description: "Sort option: most recently edited lists first.",
    },
    "my.sort.recent.hint": {
        text: "Latest edits first",
        description: "Explains the Recently updated sort, under its name in the menu.",
    },
    "my.sort.newest": {
        text: "Newest",
        description: "Sort option: most recently created lists first.",
    },
    "my.sort.newest.hint": {
        text: "Most recently created",
        description: "Explains the Newest sort, under its name in the menu.",
    },
    "my.sort.alpha": {
        text: "Alphabetical",
        description: "Sort option: ordered by title.",
    },
    "my.sort.alpha.hint": {
        text: "A → Z by title",
        description: "Explains the Alphabetical sort. Name the first and last letters of your own alphabet; keep the arrow.",
    },
    "my.sort.views": {
        text: "Most viewed",
        description: "Sort option: highest all-time view count first.",
    },
    "my.sort.views.hint": {
        text: "All-time view count",
        description: "Explains the Most viewed sort, under its name in the menu.",
    },
    "my.sort.favorites": {
        text: "Most favorited",
        description: "Sort option: most-saved lists first.",
    },
    "my.sort.favorites.hint": {
        text: "All-time favorites",
        description: "Explains the Most favorited sort, under its name in the menu.",
    },
    "my.type.label": {
        text: "Filter by list type",
        description: "Accessible name of the tab strip choosing which kind of the player's lists is shown.",
    },
    "my.type.all": {
        text: "All",
        description: "Tab showing every list the player has.",
    },
    "my.type.community": {
        text: "Community",
        description: "Tab showing only the lists the player built themselves.",
    },
    "my.type.official": {
        text: "Official",
        description: "Tab showing only the official lists the player maintains for the team.",
    },
    "my.sortLabel": {
        text: "Sort",
        description: "Small uppercase label inside the sort button, before the sort in force.",
    },
    "my.sortTrigger": {
        text: "Sort: {label}",
        description: "Accessible name of the sort menu's button, naming the sort in force.",
    },
    "my.view.label": {
        text: "View mode",
        description: "Accessible name of the pair of buttons choosing between the card grid and the compact list.",
    },
    "my.view.grid": {
        text: "Grid view",
        description: "Accessible name of the button that shows the lists as cards.",
    },
    "my.view.list": {
        text: "List view",
        description: "Accessible name of the button that shows the lists as compact rows.",
    },
    "my.search.placeholder": {
        text: "Search your lists...",
        description: "Placeholder in the box that filters the player's own lists. Keep the three dots.",
    },
    "my.search.label": {
        text: "Search your tier lists",
        description: "Accessible name of the search box.",
    },
    "my.search.clear": {
        text: "Clear search",
        description: "Accessible name of the button that empties the search box.",
    },
    "my.results.count": {
        text: "{shown} of {total}",
        description: "Result count under the filters on the player's own lists, e.g. '3 of 12'. Both numbers are styled separately and may move wherever the phrase needs them.",
    },
    "my.results.matching": {
        text: "· matching {query}",
        description: "Names the active search term after the result count, e.g. '· matching \"meta\"'. {query} is the term in quotes and may move; keep the middle dot.",
    },
} satisfies MessageMap;

// `dynamic`: the sort option names and hints are stored in the MY_SORT_OPTIONS
// table and resolved as `t(opt.labelKey)`, so the extractor has no literal
// call site to match them against.
export const { keys } = defineMessages({ namespace, messages, dynamic: true });
