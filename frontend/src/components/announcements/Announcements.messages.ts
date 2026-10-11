import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

/** The game news page. */
export const namespace = "announcements";

export const messages = {
    "page.title": {
        text: "Game news",
        description: "Heading of the page that mirrors the in-game notice board (events, banners, maintenance notices) from the official servers.",
    },
    "page.description": {
        text: "The in-game notice board, straight from the official servers. Checked every few minutes.",
        description: "Subtitle under the game news heading.",
    },
    "server.label": {
        text: "Server",
        description: "Accessible name of the server tabs on the game news page. Each tab shows that game server's own notice board.",
    },
    "search.placeholder": {
        text: "Search news",
        description: "Placeholder of the search box that filters the game news list by title.",
    },
    "search.label": {
        text: "Search game news by title",
        description: "Accessible name of the game news search box.",
    },
    "filter.label": {
        text: "Category",
        description: "Accessible name of the row of category chips that filter the game news list.",
    },
    "filter.all": {
        text: "All",
        description: "Category chip that shows every notice.",
    },
    "group.activity": {
        text: "Events",
        description: "Category name for the board's ACTIVITY notices: events, banners, sales.",
    },
    "group.system": {
        text: "Notices",
        description: "Category name for the board's SYSTEM notices: maintenance, official statements.",
    },
    "list.label": {
        text: "News on this server",
        description: "Accessible name of the list of game news entries.",
    },
    "list.unread": {
        text: "Unread",
        description: "Screen-reader text next to a news entry the reader hasn't opened yet. Visually it is a dot.",
    },
    "list.markAllRead": {
        text: "Mark all read",
        description: "Button that marks every notice on the current server as read.",
    },
    "list.count": {
        text: "{count, plural, one {# notice} other {# notices}}",
        description: "How many notices the filtered list shows.",
    },
    "list.checked": {
        text: "Checked {time}",
        description: "When the site last read the official board. {time} is a relative time like '3 minutes ago'.",
    },
    "empty.title": {
        text: "Nothing on the board",
        description: "Empty state when the server's notice board has no notices, or hasn't been read yet.",
    },
    "empty.body": {
        text: "The board is empty right now, or we haven't read it yet. Check back in a few minutes.",
        description: "Body of the empty state for a notice board with nothing on it.",
    },
    "empty.filtered.title": {
        text: "No matches",
        description: "Empty state when the search or category filter hides every notice.",
    },
    "empty.filtered.body": {
        text: "Nothing matches that. Try another word or category.",
        description: "Body of the empty state when the filters hide every notice.",
    },
    "empty.filtered.clear": {
        text: "Clear filters",
        description: "Button in the no-matches state that clears the search and the category.",
    },
    "error.title": {
        text: "Couldn't load the news",
        description: "Error state when the game news list or one notice fails to load.",
    },
    "error.body": {
        text: "Something went wrong reaching the server. Give it another try.",
        description: "Body of the game news error state.",
    },
    "error.retry": {
        text: "Try again",
        description: "Button in the game news error state that retries the request.",
    },
    "detail.pick": {
        text: "Pick a notice to read it here.",
        description: "Placeholder in the reading pane on wide screens before any notice is chosen.",
    },
    "detail.back": {
        text: "All news",
        description: "Link back from one notice to the news list, shown on phones where the list and the notice don't fit side by side.",
    },
    "detail.official": {
        text: "Open the official notice",
        description: "Link to the notice on the official game site, opening in a new tab.",
    },
    "detail.gone.title": {
        text: "This notice is gone",
        description: "Shown when a linked notice is no longer on the official board.",
    },
    "detail.gone.body": {
        text: "It was taken off the board. The rest of the news is still here.",
        description: "Body shown when a linked notice is no longer on the official board.",
    },
    "detail.bannerAlt": {
        text: "Banner for {title}",
        description: "Alt text of a notice's banner image. {title} is the notice title.",
    },
    "detail.imageOnly": {
        text: "This notice is just the banner above.",
        description: "Shown under a notice that has a banner image and no text.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
