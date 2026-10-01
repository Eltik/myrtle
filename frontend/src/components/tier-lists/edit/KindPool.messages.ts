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
    "edit.pool.dropArea.any": {
        text: "Drag a tile here to unplace it",
        description: "Accessible name of the pool area for any kind other than operators; the area doubles as the target for taking a tile off the board.",
    },
    "edit.pool.kicker.class": {
        text: "Class pool",
        description: "Small uppercase label above the pool of classes that can be dragged onto a tier; also the title of the larger pool dialog.",
    },
    "edit.pool.kicker.subclass": {
        text: "Subclass pool",
        description: "Small uppercase label above the pool of subclasses that can be dragged onto a tier; also the title of the larger pool dialog.",
    },
    "edit.pool.kicker.faction": {
        text: "Faction pool",
        description: "Small uppercase label above the pool of factions that can be dragged onto a tier; also the title of the larger pool dialog.",
    },
    "edit.pool.kicker.enemy": {
        text: "Enemy pool",
        description: "Small uppercase label above the pool of enemies that can be dragged onto a tier; also the title of the larger pool dialog.",
    },
    "edit.pool.kicker.event": {
        text: "Event pool",
        description: "Small uppercase label above the pool of events that can be dragged onto a tier; also the title of the larger pool dialog.",
    },
    "edit.pool.kicker.stronghold_bond": {
        text: "Stronghold bond pool",
        description: "Small uppercase label above the pool of Stronghold bonds that can be dragged onto a tier; also the title of the larger pool dialog.",
    },
    "edit.pool.search.class": {
        text: "Search classes",
        description: "Accessible name of the pool's search box while it lists classes.",
    },
    "edit.pool.search.subclass": {
        text: "Search subclasses",
        description: "Accessible name of the pool's search box while it lists subclasses.",
    },
    "edit.pool.search.faction": {
        text: "Search factions",
        description: "Accessible name of the pool's search box while it lists factions.",
    },
    "edit.pool.search.enemy": {
        text: "Search enemies",
        description: "Accessible name of the pool's search box while it lists enemies.",
    },
    "edit.pool.search.event": {
        text: "Search events",
        description: "Accessible name of the pool's search box while it lists events.",
    },
    "edit.pool.search.stronghold_bond": {
        text: "Search Stronghold bonds",
        description: "Accessible name of the pool's search box while it lists Stronghold bonds.",
    },
    "edit.pool.grid.class": {
        text: "Available classes",
        description: "Accessible name of the grid of classes tiles in the pool.",
    },
    "edit.pool.grid.subclass": {
        text: "Available subclasses",
        description: "Accessible name of the grid of subclasses tiles in the pool.",
    },
    "edit.pool.grid.faction": {
        text: "Available factions",
        description: "Accessible name of the grid of factions tiles in the pool.",
    },
    "edit.pool.grid.enemy": {
        text: "Available enemies",
        description: "Accessible name of the grid of enemies tiles in the pool.",
    },
    "edit.pool.grid.event": {
        text: "Available events",
        description: "Accessible name of the grid of events tiles in the pool.",
    },
    "edit.pool.grid.stronghold_bond": {
        text: "Available Stronghold bonds",
        description: "Accessible name of the grid of Stronghold bonds tiles in the pool.",
    },
    "edit.pool.level": {
        text: "Level",
        description: "Label of the faction-level filter (nation, group, team) in the pool dialog. Rendered uppercase.",
    },
    "edit.pool.level.group": {
        text: "Filter by level",
        description: "Accessible name of the row of faction-level filter buttons.",
    },
    "edit.pool.rank": {
        text: "Rank",
        description: "Label of the enemy-rank filter (normal, elite, leader) in the pool dialog. Rendered uppercase.",
    },
    "edit.pool.rank.group": {
        text: "Filter by rank",
        description: "Accessible name of the row of enemy-rank filter buttons.",
    },
    "edit.pool.type": {
        text: "Type",
        description: "Label of the type filter (event category, or Stronghold bond type) in the pool dialog. Rendered uppercase.",
    },
    "edit.pool.type.group": {
        text: "Filter by type",
        description: "Accessible name of the row of type filter buttons.",
    },
    "edit.pool.edition": {
        text: "Edition",
        description: "Label of the filter that tells an event's first run from its reruns, in the pool dialog. Rendered uppercase.",
    },
    "edit.pool.edition.group": {
        text: "Filter by edition",
        description: "Accessible name of the row of edition filter buttons.",
    },
    "edit.pool.edition.original": {
        text: "Original",
        description: "Edition filter option: the event's first run, as opposed to a rerun.",
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
