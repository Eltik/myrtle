import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tierLists";

export const messages = {
    "edit.pool.kicker": {
        text: "Operator pool",
        description: "Small uppercase label above the list of operators that can be dragged onto a tier.",
    },
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
    "edit.pool.search.label": {
        text: "Search operators",
        description: "Accessible name of the pool's search box.",
    },
    "edit.pool.search.placeholder": {
        text: "Search by name…",
        description: "Placeholder in the pool's search box. Keep the single-character ellipsis.",
    },
    "edit.pool.dropArea": {
        text: "Drag operators here to unplace them",
        description: "Accessible name of the pool area, which doubles as the target for taking an operator off the board.",
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
    "edit.pool.gridLabel": {
        text: "Available operators",
        description: "Accessible name of the grid of operator tiles in the pool.",
    },
    "edit.pool.hintDragging": {
        text: "Drop on a tier to place, or release here to unplace.",
        description: "Footer hint under the pool while an operator is being dragged.",
    },
    "edit.pool.hintIdle": {
        text: "Drag a tile onto a tier, or tap to pick one.",
        description: "Footer hint under the pool when nothing is being dragged.",
    },
    "edit.pool.dialogTitle": {
        text: "Operator pool",
        description: "Title of the larger operator-pool dialog.",
    },
    "edit.pool.dialogSub": {
        text: "{matched} of {total} available. Tap a tile to place it on a tier.",
        description: "Subtitle of the operator-pool dialog, e.g. '120 of 340 available. Tap a tile to place it on a tier.'. {matched} and {total} are the two counts, each styled separately, and may move wherever the sentence needs them.",
    },
    "edit.pool.rarity": {
        text: "Rarity",
        description: "Label of the rarity filter in the pool dialog. Rendered uppercase.",
    },
    "edit.pool.rarity.group": {
        text: "Filter by rarity",
        description: "Accessible name of the row of rarity filter buttons.",
    },
    "edit.pool.rarity.option": {
        text: "{rarity} star",
        description: "Accessible name of one rarity filter button, e.g. '6 star'.",
    },
    "edit.pool.class": {
        text: "Class",
        description: "Label of the class filter in the pool dialog. Rendered uppercase.",
    },
    "edit.pool.class.group": {
        text: "Filter by class",
        description: "Accessible name of the row of class filter buttons.",
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
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
