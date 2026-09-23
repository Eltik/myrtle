import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "user";

export const messages = {
    "search.sort.scoped": {
        text: "{label} operators",
        description: "Rank-by label when ranking by operators of one class or archetype. {label} is the game's class or archetype name, e.g. 'Guard operators', 'Centurion operators'.",
    },
    "search.toolbar.rankBy": {
        text: "Rank by",
        description: "Small caps kicker in front of the active sort name on the rank-by trigger.",
    },
    "search.toolbar.rankBy.aria": {
        text: "Rank players by",
        description: "Accessible name of the rank-by trigger button.",
    },
    "search.toolbar.rankBy.group.metric": {
        text: "Roster metric",
        description: "Heading over the eight metric chips inside the rank-by popup.",
    },
    "search.toolbar.rankBy.group.scope": {
        text: "Operators of a class",
        description: "Heading over the class icons inside the rank-by popup: pick a class or archetype to rank by how many of them a player owns.",
    },
    "search.toolbar.dir.aria": {
        text: "Sort direction",
        description: "Accessible name of the button that flips the sort between ascending and descending.",
    },
    "search.toolbar.dir.desc": {
        text: "High to low",
        description: "Visible label of the direction button while a count sort descends.",
    },
    "search.toolbar.dir.asc": {
        text: "Low to high",
        description: "Visible label of the direction button while a count sort ascends.",
    },
    "search.toolbar.dir.joined.asc": {
        text: "Oldest first",
        description: "Visible label of the direction button while the 'joined' sort ascends (earliest accounts first).",
    },
    "search.toolbar.dir.joined.desc": {
        text: "Newest first",
        description: "Visible label of the direction button while the 'joined' sort descends (latest accounts first).",
    },
    "search.toolbar.filters": {
        text: "Filters",
        description: "Small caps kicker in front of the active-filter count above the filter controls.",
    },
    "search.toolbar.filters.count": {
        text: "{count, plural, =0 {none active} one {# active} other {# active}}",
        description: "How many of the three roster filters are set, after the 'Filters' kicker.",
    },
    "search.toolbar.clearFilters": {
        text: "Clear filters",
        description: "Button that unsets every roster filter. Shown in the toolbar while any is set, and in the empty state when the filters match nobody.",
    },
    "search.toolbar.has.label": {
        text: "Owns operators",
        description: "Label of the picker for operators every listed player must own. Several can be chosen.",
    },
    "search.toolbar.has.placeholder": {
        text: "Add an operator…",
        description: "Placeholder in the 'owns operators' picker while it is empty. Keep the single-character ellipsis.",
    },
    "search.toolbar.support.label": {
        text: "Support unit has",
        description: "Label of the picker for one operator that must be in the player's support unit (the operators they lend to friends).",
    },
    "search.toolbar.support.placeholder": {
        text: "Pick an operator…",
        description: "Placeholder in the 'support unit has' picker while it is empty. Keep the single-character ellipsis.",
    },
    "search.toolbar.all.label": {
        text: "Owns every",
        description: "Small caps kicker on the trigger for the filter that requires the whole of a class or archetype to be owned. The chosen class or archetype name follows.",
    },
    "search.toolbar.all.aria": {
        text: "Owns every operator of a class or archetype",
        description: "Accessible name of the trigger for the complete-class filter.",
    },
    "search.toolbar.all.none": {
        text: "Not set",
        description: "Value on the complete-class filter trigger while no class or archetype is chosen.",
    },
    "search.toolbar.all.hint": {
        text: "Only players who own every obtainable operator of the chosen scope on their own server.",
        description: "Explanation at the top of the complete-class filter popup.",
    },
    "search.toolbar.all.clear": {
        text: "Clear",
        description: "Button inside the complete-class filter popup that unsets the filter.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
