import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "grids";

export const messages = {
    "browse.kicker": {
        text: "Community",
        description: "Small uppercase label above the grid browser's heading.",
    },
    "browse.title": {
        text: "Grids",
        description: "Heading of the grid browser. A grid is a board of labelled cells ('Favorite Vanguard') each filled with an operator, skin or other pick.",
    },
    "browse.blurb": {
        text: "Fill a board of prompts with your picks, share it, or use someone else's as a template.",
        description: "Line under the grid browser's heading.",
    },
    "browse.mine": {
        text: "My grids",
        description: "Button to the signed-in user's own grids.",
    },
    "browse.searchLabel": {
        text: "Search grids",
        description: "Accessible name of the grid browser's search box.",
    },
    "browse.searchPlaceholder": {
        text: "Search by title",
        description: "Placeholder of the grid browser's search box.",
    },
    "browse.sortLabel": {
        text: "Sort grids",
        description: "Accessible name of the Recent / Popular toggle.",
    },
    "browse.sort.recent": {
        text: "Recent",
        description: "Sort option: most recently updated grids first.",
    },
    "browse.sort.popular": {
        text: "Popular",
        description: "Sort option: grids used most as templates first.",
    },
    "browse.error": {
        text: "Grids could not be loaded.",
        description: "Shown when the grid list failed to load.",
    },
    "browse.retry": {
        text: "Try again",
        description: "Reloads the grid list after an error.",
    },
    "browse.empty": {
        text: "No grids yet.",
        description: "Shown when no public grid exists.",
    },
    "browse.emptySearch": {
        text: "No grids match your search.",
        description: "Shown when the search leaves no grid.",
    },
    "browse.emptyHint": {
        text: "Start one with New grid.",
        description: "Hint under an empty grid list. 'New grid' is the create button's label.",
    },
    "browse.pagination": {
        text: "Pages",
        description: "Accessible name of the pagination under the grid list.",
    },
    "browse.prev": {
        text: "Previous",
        description: "Goes to the previous page of grids.",
    },
    "browse.next": {
        text: "Next",
        description: "Goes to the next page of grids.",
    },
    "browse.pageOf": {
        text: "Page {page} of {total}",
        description: "Position in the grid list's pages. Both numbers are already formatted.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
