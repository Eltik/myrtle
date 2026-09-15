import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tools";

export const messages = {
    "randomizer.roster.search": {
        text: "Search operators…",
        description: "Placeholder in the roster search box. Keep the single ellipsis character.",
    },
    "randomizer.roster.clearSearch": {
        text: "Clear search",
        description: "Accessible name of the small cross that empties the search box.",
    },
    "randomizer.roster.inView": {
        text: "in view",
        description: "Follows a 'selected / shown' count when filters are hiding part of the roster, e.g. '12 / 40 in view'. Lowercase; set in small caps.",
    },
    "randomizer.roster.selected": {
        text: "selected",
        description: "Follows a 'selected / total' count when nothing is hidden, e.g. '12 / 40 selected'. Lowercase; set in small caps.",
    },
    "randomizer.roster.totalInRoster": {
        text: "{count} total in roster",
        description: "Second line under the count, giving the roster size across every filter.",
    },
    "randomizer.roster.all": {
        text: "All",
        description: "Button that selects every operator currently shown. Very narrow button.",
    },
    "randomizer.roster.none": {
        text: "None",
        description: "Button that deselects every operator currently shown. Very narrow button.",
    },
    "randomizer.roster.reset": {
        text: "Reset",
        description: "Button that restores the default roster.",
    },
    "randomizer.roster.reset.title": {
        text: "Restore the default state (every operator selected).",
        description: "Native tooltip on the Reset button.",
    },
    "randomizer.roster.syncProfile": {
        text: "Sync profile ({count})",
        description: "Button that replaces the roster with the operators the player owns; the count is how many that is.",
    },
    "randomizer.roster.groupCount": {
        text: "{selected} / {total}",
        description: "How many operators of one star rating are selected, e.g. '8 / 34'.",
    },
    "randomizer.roster.groupClear": {
        text: "Clear",
        description: "Button that deselects every operator of one star rating. Very narrow button.",
    },
    "randomizer.roster.groupAll": {
        text: "All",
        description: "Button that selects every operator of one star rating. Very narrow button.",
    },
    "randomizer.roster.groupClear.aria": {
        text: "Clear all {rarity} operators",
        description: "Accessible name of the Clear button on one star-rating group. {rarity} is a rating label such as '6★'.",
    },
    "randomizer.roster.groupAll.aria": {
        text: "Select all {rarity} operators",
        description: "Accessible name of the All button on one star-rating group. {rarity} is a rating label such as '6★'.",
    },
    "randomizer.roster.hint": {
        text: "Showing operators that match your Operators filters. {count} hidden by {filters}.",
        description: "Note above the roster. {count} is the number hidden, in a brighter colour, and {filters} the list of active filters (or randomizer.roster.hintFilters when none can be named); both may move wherever the sentence needs them. 'Operators' names the tab of the same name.",
    },
    "randomizer.roster.hintFilters": {
        text: "active filters",
        description: "Stands in for the list of filters when none can be named.",
    },
    "randomizer.roster.filter.e2": {
        text: "E2 only",
        description: "Named in the note as one of the active filters. 'E2' is the game's second Elite promotion.",
    },
    "randomizer.roster.filter.owned": {
        text: "Owned only",
        description: "Named in the note as one of the active filters: only operators the player owns.",
    },
    "randomizer.roster.rarityChip": {
        text: "{rarity}★",
        description: "One star rating in the list of filters, e.g. '3★'.",
    },
    "randomizer.roster.empty.title": {
        text: "No operators to show",
        description: "Heading when the roster list is empty.",
    },
    "randomizer.roster.empty.search": {
        text: "No operators match your search.",
        description: "Body when the roster list is empty because of the search box.",
    },
    "randomizer.roster.empty.filters": {
        text: "All operators are filtered out by your Operators tab settings.",
        description: "Body when the roster list is empty because of the other tab's filters. 'Operators' names the tab of the same name.",
    },
    "randomizer.roster.empty.none": {
        text: "There are no operators available.",
        description: "Body when the roster list is empty and no filter explains it.",
    },
    "randomizer.roster.empty.reset": {
        text: "Reset roster",
        description: "Button in the empty state that restores the default roster.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
