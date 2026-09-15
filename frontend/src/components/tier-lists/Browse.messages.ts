import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tierLists";

export const messages = {
    "browse.error": {
        text: "Failed to load tier lists.",
        description: "Shown in place of the grid when the tier lists could not be fetched.",
    },
    "browse.retry": {
        text: "Retry",
        description: "Button that re-runs the failed request for the tier lists.",
    },
    "browse.retrying": {
        text: "Retrying…",
        description: "The retry button's label while the request is in flight. Keep the single-character ellipsis.",
    },
    "browse.favorites.emptyTitle": {
        text: "No favorites yet",
        description: "Empty state heading on the Favorites tab, before the visitor has saved any tier list.",
    },
    "browse.favorites.emptyBody": {
        text: "Tap the heart on any tier list to save it here for quick access.",
        description: "Empty state body on the Favorites tab, saying how to add one.",
    },
    "browse.favorites.emptyAction": {
        text: "Browse tier lists",
        description: "Button in the Favorites empty state that switches back to all tier lists.",
    },
    "browse.filtered.emptyTitle": {
        text: "No lists match these filters.",
        description: "Empty state heading when the active filters leave no tier list in the grid.",
    },
    "browse.filtered.emptyBody": {
        text: "Try clearing flairs or switching the type filter.",
        description: "Empty state body suggesting how to get results back. 'Flairs' are the topic tags on a list.",
    },
    "browse.filtered.clear": {
        text: "Clear filters",
        description: "Button that resets the search box, type tab and flair selection.",
    },
    "browse.loadMore": {
        text: "Load {count} more",
        description: "Button under the grid that appends the next page of tier lists.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
