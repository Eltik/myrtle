import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tierLists";

export const messages = {
    "browse.sort.trending": {
        text: "Trending",
        description: "Sort option: lists getting the most attention right now.",
    },
    "browse.sort.trending.hint": {
        text: "Hot in the last 24h",
        description: "Explains the Trending sort, under its name in the menu.",
    },
    "browse.sort.recent": {
        text: "Recently updated",
        description: "Sort option: most recently edited lists first.",
    },
    "browse.sort.recent.hint": {
        text: "Latest edits first",
        description: "Explains the Recently updated sort, under its name in the menu.",
    },
    "browse.sort.newest": {
        text: "Newest",
        description: "Sort option: most recently created lists first.",
    },
    "browse.sort.newest.hint": {
        text: "Most recently created",
        description: "Explains the Newest sort, under its name in the menu.",
    },
    "browse.sort.views": {
        text: "Most viewed",
        description: "Sort option: highest all-time view count first.",
    },
    "browse.sort.views.hint": {
        text: "All-time view count",
        description: "Explains the Most viewed sort, under its name in the menu.",
    },
    "browse.sort.favorites": {
        text: "Most favorited",
        description: "Sort option: most-saved lists first.",
    },
    "browse.sort.favorites.hint": {
        text: "All-time favorites",
        description: "Explains the Most favorited sort, under its name in the menu.",
    },
    "browse.sort.shares": {
        text: "Most shared",
        description: "Sort option: most-shared lists first.",
    },
    "browse.sort.shares.hint": {
        text: "All-time shares",
        description: "Explains the Most shared sort, under its name in the menu.",
    },
    "browse.type.label": {
        text: "Tier list type",
        description: "Accessible name of the tab strip choosing which kind of list is listed.",
    },
    "browse.type.all": {
        text: "All",
        description: "Tab listing every tier list, official and community alike.",
    },
    "browse.type.official": {
        text: "Official",
        description: "Tab listing only the lists maintained by the site's own team.",
    },
    "browse.type.community": {
        text: "Community",
        description: "Tab listing only the lists built by players.",
    },
    "browse.type.favorites": {
        text: "Favorites",
        description: "Tab listing only the lists the signed-in visitor has saved.",
    },
    "browse.sortTrigger": {
        text: "Sort: {label}",
        description: "Accessible name of the sort menu's button, naming the sort in force.",
    },
    "browse.flairs": {
        text: "Flairs",
        description: "Button that shows or hides the row of topic tags a list can carry.",
    },
    "browse.flair.chip": {
        text: "Flair {label}, {count, plural, one {# list} other {# lists}}",
        description: "Accessible name of one flair chip, naming the tag and how many lists carry it. {label} is the tag's own name.",
    },
    "browse.flair.chipEmpty": {
        text: "Flair {label}, no lists",
        description: "Accessible name of a flair chip no list currently carries. {label} is the tag's own name.",
    },
    "browse.flair.clear": {
        text: "Clear",
        description: "Chip at the end of the flair row that deselects every flair.",
    },
    "browse.search.placeholder": {
        text: "Search lists...",
        description: "Placeholder in the box that filters lists by title, description or author. Keep the three dots.",
    },
    "browse.search.label": {
        text: "Search tier lists",
        description: "Accessible name of the search box.",
    },
    "browse.search.clear": {
        text: "Clear search",
        description: "Accessible name of the button that empties the search box.",
    },
    "browse.results.count": {
        text: "{shown} of {total}",
        description: "Result count under the browse filters, e.g. '12 of 80'. Both numbers are styled separately and may move wherever the phrase needs them.",
    },
    "browse.results.matching": {
        text: "· matching {query}",
        description: "Names the active search term after the result count, e.g. '· matching \"texas\"'. {query} is the term in quotes and may move; keep the middle dot.",
    },
} satisfies MessageMap;

// `dynamic`: the sort option names and hints are stored in the SORT_OPTIONS
// table and resolved as `t(opt.labelKey)`, so the extractor has no literal
// call site to match them against.
export const { keys } = defineMessages({ namespace, messages, dynamic: true });
