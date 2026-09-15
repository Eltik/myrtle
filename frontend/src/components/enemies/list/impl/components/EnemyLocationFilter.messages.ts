import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "enemies";

export const messages = {
    "location.trigger": {
        text: "Appears In",
        description: "Label on the button that opens the location filter, i.e. which stages an enemy shows up in.",
    },
    "location.trigger.aria": {
        text: "Filter by where enemies appear",
        description: "Accessible name of the 'Appears In' filter button, spelling out what the short label means.",
    },
    "location.search.placeholder": {
        text: "Search events, modes, or stages…",
        description: "Placeholder in the location filter's search box. Ends with a single ellipsis character.",
    },
    "location.search.aria": {
        text: "Search locations",
        description: "Accessible name of the location filter's search box.",
    },
    "location.search.clear": {
        text: "Clear search",
        description: "Accessible name of the small x button that empties the location filter's search box.",
    },
    "location.empty": {
        text: "No matching locations.",
        description: "Empty state inside the location filter when the search matches no event, mode or stage.",
    },
    "location.selectAll": {
        text: "Select all {name}",
        description: "Accessible name of a checkbox that selects every stage under a category or event. {name} is the category or event name, which is game data.",
    },
    "location.deselectAll": {
        text: "Deselect all {name}",
        description: "Accessible name of that same checkbox when everything under it is already selected.",
    },
    "location.selected": {
        text: "{count} selected",
        description: "Footer of the location filter, counting how many locations are picked.",
    },
    "location.clearAll": {
        text: "Clear all",
        description: "Button in the location filter's footer that unselects every location.",
    },
    "location.remove": {
        text: "Remove {name}",
        description: "Accessible name of the x on a selected-location chip. {name} is the event or stage name, which is game data.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
