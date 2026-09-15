import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "enemies";

export const messages = {
    "list.breadcrumb": {
        text: "Breadcrumb",
        description: "Accessible name of the breadcrumb <nav> landmark at the top of the enemy list.",
    },
    "list.breadcrumb.collection": {
        text: "Collection",
        description: "First breadcrumb crumb, naming the section the enemy list lives in.",
    },
    "list.breadcrumb.enemies": {
        text: "Enemies",
        description: "Last breadcrumb crumb: this page.",
    },
    "list.title": {
        text: "Enemy Database",
        description: "Page heading of the enemy list.",
    },
    "list.blurb": {
        text: "View all {count} enemies catalogued in Arknights, sorted by Hypergryph's internal sort index by default.",
        description: "Paragraph under the enemy-list heading. {count} is the bold number of enemies and may move wherever the sentence needs it. 'Arknights' and 'Hypergryph' are the game and its developer; keep both names.",
    },
    "list.results.aria": {
        text: "Enemy results",
        description: "Accessible name of the <main> landmark holding the filters, the result grid and the pager.",
    },
    "list.search.placeholder": {
        text: "Search by name, callsign, or race…",
        description: "Placeholder in the enemy search box. 'Callsign' is the enemy's handbook index, e.g. 'B1'. Ends with a single ellipsis character.",
    },
    "list.search.aria": {
        text: "Search enemies",
        description: "Accessible name of the enemy search box.",
    },
    "list.viewMode.aria": {
        text: "View mode",
        description: "Accessible name of the grid/list toggle button group.",
    },
    "list.viewMode.grid": {
        text: "Grid",
        description: "Grid view button: its tooltip and its hover title. Shows enemies as portrait cards.",
    },
    "list.viewMode.list": {
        text: "List",
        description: "List view button: its tooltip and its hover title. Shows enemies as compact rows.",
    },
    "list.sort.label": {
        text: "Sort",
        description: "Uppercase prefix inside the sort select, before the chosen sort option. Very tight space.",
    },
    "list.sort.aria": {
        text: "Sort enemies",
        description: "Accessible name of the sort select.",
    },
    "list.sort.asc": {
        text: "Ascending",
        description: "Hover title of the sort-direction button when the current order is ascending.",
    },
    "list.sort.desc": {
        text: "Descending",
        description: "Hover title of the sort-direction button when the current order is descending.",
    },
    "list.sort.toggle": {
        text: "Toggle sort direction",
        description: "Accessible name of the button that flips between ascending and descending.",
    },
    "list.perPage.label": {
        text: "Show",
        description: "Uppercase prefix inside the page-size select, before the number of rows. Very tight space.",
    },
    "list.perPage.aria": {
        text: "Items per page",
        description: "Accessible name of the page-size select.",
    },
    "list.perPage.all": {
        text: "All",
        description: "Page-size option: put every result on one page.",
    },
    "list.export": {
        text: "Export",
        description: "Label on the export button, hidden on phone widths where only the icon shows.",
    },
    "list.export.aria": {
        text: "Export enemies",
        description: "Accessible name of the export button, and its tooltip.",
    },
    "list.export.dialogTitle": {
        text: "Enemies",
        description: "Headline of the export dialog, naming what is being exported.",
    },
    "list.showing": {
        text: "Showing {from}-{to} of {total} enemies",
        description: "Result count above the pager, e.g. 'Showing 1-48 of 412 enemies'. All three numbers are bold and may move wherever the sentence needs them. No closing full stop: a link may follow after a middle dot.",
    },
    "list.clearFilters": {
        text: "{count, plural, one {Clear # filter} other {Clear # filters}}",
        description: "Inline link after the result count that drops every active filter, counting how many are on.",
    },
    "list.hint": {
        text: "Click a card for details",
        description: "Hint beside the pager, hidden on phone widths, telling the reader a card opens the enemy's page.",
    },
    "list.col.name": {
        text: "Name",
        description: "List-view column header: the enemy's name and callsign.",
    },
    "list.col.threat": {
        text: "Threat",
        description: "List-view column header: the Normal / Elite / Boss tier.",
    },
    "list.col.damage": {
        text: "Damage",
        description: "List-view column header: what damage type the enemy deals.",
    },
    "list.col.hp": {
        text: "HP",
        description: "List-view column header: hit points. Abbreviated to fit a narrow, right-aligned column.",
    },
    "list.empty.title": {
        text: "No enemies match your filters",
        description: "Empty-state heading when the filters and search match no enemy.",
    },
    "list.empty.body": {
        text: "Adjust the chip rail or your search above, or clear everything and start fresh.",
        description: "Empty-state paragraph. 'Chip rail' is the row of filter chips above the results.",
    },
    "list.empty.clear": {
        text: "Clear all filters",
        description: "Empty-state button that drops every active filter.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
