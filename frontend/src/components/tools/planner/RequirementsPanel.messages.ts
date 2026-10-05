import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tools";

export const messages = {
    "planner.req.title": {
        text: "Requirements",
        description: "Heading of the panel listing everything the selected plans need.",
    },
    "planner.req.search": {
        text: "Search requirements...",
        description: "Placeholder in the requirement search box. Three full stops, not an ellipsis character.",
    },
    "planner.req.view": {
        text: "Requirements view",
        description: "Accessible name of the group of buttons choosing how requirements are laid out.",
    },
    "planner.req.view.grouped": {
        text: "Grouped",
        description: "Native tooltip on the button that groups requirements by item category.",
    },
    "planner.req.view.grouped.aria": {
        text: "Grouped view",
        description: "Accessible name of the button that groups requirements by item category.",
    },
    "planner.req.view.flat": {
        text: "Flat",
        description: "Native tooltip on the button that shows requirements as one ungrouped list.",
    },
    "planner.req.view.flat.aria": {
        text: "Flat view",
        description: "Accessible name of the button that shows requirements as one ungrouped list.",
    },
    "planner.req.view.byOperator": {
        text: "By operator",
        description: "Native tooltip on the button that shows one requirement list per operator.",
    },
    "planner.req.view.byOperator.aria": {
        text: "By operator view",
        description: "Accessible name of the button that shows one requirement list per operator.",
    },
    "planner.req.filter.all": {
        text: "All",
        description: "Filter option that applies no restriction, in both the type and status dropdowns.",
    },
    "planner.req.filter.type": {
        text: "Type",
        description: "Placeholder in the item-type filter before a type is chosen.",
    },
    "planner.req.filter.typeValue": {
        text: "Type: {value}",
        description: "Closed state of the item-type filter, e.g. 'Type: Materials'. Keep the colon.",
    },
    "planner.req.filter.status": {
        text: "Status",
        description: "Placeholder in the status filter before a status is chosen.",
    },
    "planner.req.filter.statusValue": {
        text: "Status: {value}",
        description: "Closed state of the status filter, e.g. 'Status: Missing'. Keep the colon.",
    },
    "planner.req.col.item": {
        text: "Item",
        description: "Table heading over the material names.",
    },
    "planner.req.col.required": {
        text: "Required",
        description: "Table heading over how many of each item the plans need.",
    },
    "planner.req.col.have": {
        text: "Have",
        description: "Table heading over how many of each item the player already owns.",
    },
    "planner.req.col.craftable": {
        text: "Craftable",
        description: "Table heading over how many of each item could be crafted from what the player owns.",
    },
    "planner.req.col.missing": {
        text: "Missing",
        description: "Table heading over how many of each item are still unaccounted for.",
    },
    "planner.req.craft": {
        text: "craft {count}",
        description: "Shown in the Missing column when the shortfall is craftable. Lowercase on purpose; the column is narrow.",
    },
    "planner.req.craftTitle": {
        text: "You have {have} of {required} - the remaining {shortfall} must be crafted",
        description: "Native tooltip on a craftable shortfall. All three numbers are item counts; the dash is a plain hyphen.",
    },
    "planner.req.noneForPlans": {
        text: "No requirements for the selected plans.",
        description: "Shown in place of the table when the selected plans need nothing.",
    },
    "planner.req.noneMatching": {
        text: "No matching requirements found.",
        description: "Shown in place of the table when the search and filters leave nothing.",
    },
    "planner.req.noneMatchingOperator": {
        text: "No matching requirements.",
        description: "Shown inside one operator's section when the search and filters leave nothing for that operator.",
    },
    "planner.req.noActivePlans": {
        text: "No active plans selected.",
        description: "Shown in the by-operator view when every plan has been unselected.",
    },
    "planner.req.lastSynced": {
        text: "Last synced {time}",
        description: "Beside the Requirements heading: when the roster and inventory these numbers come from were last synced. {time} is a relative time such as '3 hours ago' or 'yesterday'.",
    },
    "planner.req.filter.tier": {
        text: "Max tier",
        description: "Placeholder in the material-tier filter before a tier is chosen. 'Tier' is the game's material rarity, T1 to T5.",
    },
    "planner.req.filter.tierValue": {
        text: "Max tier: {value}",
        description: "Closed state of the material-tier filter, e.g. 'Max tier: T3' or 'Max tier: All'. Keep the colon.",
    },
    "planner.req.filter.tierOption": {
        text: "T{tier}",
        description: "A material tier, e.g. 'T3'. The community's own shorthand, which normally stays as-is.",
    },
    "planner.req.tierNote": {
        text: "Materials above T{tier} are broken into their ingredients. LMD includes the crafting cost.",
        description: "Note under the filters while a max tier is set. LMD is the game's currency; crafting at the workshop costs it.",
    },
    "planner.req.craftGate.aria": {
        text: "Why {name} cannot be crafted yet",
        description: "Accessible name of the info button beside an item whose recipe is locked; {name} is the item name from the game data.",
    },
    "planner.req.craftGate.stages": {
        text: "Stages to clear:",
        description: "Label before the links to the stages that unlock an item's recipe. Keep the colon.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
