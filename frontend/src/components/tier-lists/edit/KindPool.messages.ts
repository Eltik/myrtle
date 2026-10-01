import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tierLists";

export const messages = {
    "edit.pool.clearFilters.label": {
        text: "Clear pool filters",
        description: "Accessible name of the button that resets the pool's search and filters.",
    },
    "edit.pool.clear": {
        text: "Clear",
        description: "Visible label of the button that resets the pool's search and filters. The button is small.",
    },
    "edit.pool.expand": {
        text: "Expand",
        description: "Button that opens the operator pool in a larger dialog.",
    },
    "edit.pool.expandHint": {
        text: "Open a larger pool view with all filters",
        description: "Tooltip on the Expand button.",
    },
    "edit.pool.search.placeholder": {
        text: "Search by name…",
        description: "Placeholder in the pool's search box. Keep the single-character ellipsis.",
    },
    "edit.pool.dropToUnplace": {
        text: "Drop to unplace",
        description: "Badge over the pool while an operator is being dragged towards it.",
    },
    "edit.pool.emptyTitle": {
        text: "No matches",
        description: "Empty state heading when no operator matches the pool's search and filters.",
    },
    "edit.pool.emptyBody": {
        text: "Try a different name or relax the filters.",
        description: "Empty state body suggesting how to get results back in the pool.",
    },
    "edit.pool.hintDragging": {
        text: "Drop on a tier to place, or release here to unplace.",
        description: "Footer hint under the pool while an operator is being dragged.",
    },
    "edit.pool.hintIdle": {
        text: "Drag a tile onto a tier, or tap to pick one.",
        description: "Footer hint under the pool when nothing is being dragged.",
    },
    "edit.pool.dialogSub": {
        text: "{matched} of {total} available. Tap a tile to place it on a tier.",
        description: "Subtitle of the operator-pool dialog, e.g. '120 of 340 available. Tap a tile to place it on a tier.'. {matched} and {total} are the two counts, each styled separately, and may move wherever the sentence needs them.",
    },
    "edit.pool.hideUsed": {
        text: "Hide already placed",
        description: "Label of the switch that keeps operators already on the board out of the pool.",
    },
    "edit.pool.placedSoFar": {
        text: "{count} placed so far",
        description: "Under the 'Hide already placed' switch: how many operators are already on the board.",
    },
    "edit.pool.nothingPlaced": {
        text: "Nothing placed yet",
        description: "Under the 'Hide already placed' switch when the board is still empty.",
    },
    "edit.pool.placed": {
        text: "Placed",
        description: "Tooltip note marking an operator already on the board. Rendered uppercase.",
    },
    "edit.pool.clearFilters": {
        text: "Clear filters",
        description: "Button in the dialog footer that resets the pool's search and filters.",
    },
    "edit.pool.done": {
        text: "Done",
        description: "Button that closes the operator-pool dialog.",
    },
    "edit.pool.tabs": {
        text: "What this pool lists",
        description: "Accessible name of the row of tabs that switch the pool between the kinds a list ranks (operators, enemies, events...).",
    },
    "edit.pool.kinds": {
        text: "Kinds",
        description: "Small button in the pool header that opens the settings for which kinds of things this list ranks.",
    },
    "edit.pool.kindsHint": {
        text: "Choose what this list ranks",
        description: "Tooltip on the Kinds button in the pool header.",
    },
    "edit.pool.loading": {
        text: "Loading…",
        description: "Shown in the pool while a kind's list is still loading. Keep the single-character ellipsis.",
    },
    "edit.pool.loadFailed": {
        text: "This list did not load.",
        description: "Shown in the pool when a kind's list failed to load.",
    },
    "edit.pool.retry": {
        text: "Try again",
        description: "Button that reloads a pool list that failed to load.",
    },
    "edit.pool.more": {
        text: "Showing {shown} of {total}. Scroll for more.",
        description: "Under a long pool grid that renders in pages: how many tiles are on screen so far out of how many match. {shown} and {total} are numbers.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
