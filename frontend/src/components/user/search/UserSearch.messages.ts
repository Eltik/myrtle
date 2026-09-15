import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "user";

export const messages = {
    "search.breadcrumb.label": {
        text: "Breadcrumb",
        description: "Accessible name of the breadcrumb <nav> above the user-search heading.",
    },
    "search.breadcrumb.doctors": {
        text: "Doctors",
        description: "First breadcrumb crumb, the section that lists players. 'Doctor' is what Arknights calls the player.",
    },
    "search.breadcrumb.current": {
        text: "Search",
        description: "Last breadcrumb crumb, naming the current page.",
    },
    "search.title": {
        text: "Search Doctors",
        description: "Page heading of the user-search page.",
    },
    "search.subtitle": {
        text: "Find Doctor profiles by nickname or browse public profiles ranked by score.",
        description: "Paragraph under the user-search page heading.",
    },
    "search.input.placeholder": {
        text: "Search by nickname…",
        description: "Placeholder in the search box. Keep the single-character ellipsis.",
    },
    "search.input.label": {
        text: "Search doctors",
        description: "Accessible name of the search box, which shows only a magnifier icon.",
    },
    "search.clearSearch": {
        text: "Clear search",
        description: "Empties the search box: accessible name of the small x button inside it, and the visible label of the button in the no-results state.",
    },
    "search.resultsFor": {
        text: "Results for",
        description: "Sits directly before the quoted search term in the status line over the results. The term follows in bold, so this fragment ends without punctuation.",
    },
    "search.quotedQuery": {
        text: '"{query}"',
        description: "The search term shown in quotation marks. Use the quotation marks your language writes.",
    },
    "search.browsing": {
        text: "Browsing public profiles by total score",
        description: "Status line over the results when the search box is empty and profiles are listed by score.",
    },
    "search.count.unit": {
        text: "{count, plural, one {doctor} other {doctors}}",
        description: "Unit after the result count, which is rendered in bold just before it: '1,234 doctors'. The number itself is not part of this string.",
    },
    "search.empty.noResults.title": {
        text: "No doctors found",
        description: "Empty-state title when a search matches no public profile.",
    },
    "search.empty.noResults.desc": {
        text: "No public profiles match {query}. Try a different nickname.",
        description: "Empty-state body when a search matches no public profile. {query} is the search term in quotes, rendered in bold by search.quotedQuery, and may move wherever the sentence needs it.",
    },
    "search.empty.none.title": {
        text: "No public profiles yet",
        description: "Empty-state title when nobody has made a profile public.",
    },
    "search.empty.none.desc": {
        text: "Public Doctor profiles will appear here as players opt in.",
        description: "Empty-state description when nobody has made a profile public.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
