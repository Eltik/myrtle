import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "operators";

export const messages = {
    "list.breadcrumb.aria": {
        text: "Breadcrumb",
        description: "Accessible name of the trail above the page title.",
    },
    "list.breadcrumb.collection": {
        text: "Collection",
        description: "First step of the breadcrumb trail: the section the operator list belongs to.",
    },
    "list.breadcrumb.operators": {
        text: "Operators",
        description: "Last step of the breadcrumb trail, naming this page.",
    },
    "list.title": {
        text: "Operators",
        description: "Page heading of the operator list.",
    },
    "list.viewAll": {
        text: "View all {count} operators.",
        description: "Paragraph under the operator-list heading. {count} is the bold number of operators and may move wherever the sentence needs it. Keep the full stop.",
    },
    "list.results.aria": {
        text: "Operator results",
        description: "Accessible name of the region holding the result grid and its toolbar.",
    },
    "list.search.placeholder": {
        text: "Search operators...",
        description: "Prompt inside the empty search box. Keep the three dots as written.",
    },
    "list.search.aria": {
        text: "Search operators",
        description: "Accessible name of the search box, which has no visible label.",
    },
    "list.viewMode.aria": {
        text: "View mode",
        description: "Accessible name of the grid/compact/list button group.",
    },
    "list.viewMode.grid": {
        text: "Grid",
        description: "View-mode option: large portrait cards. Also its tooltip.",
    },
    "list.viewMode.compact": {
        text: "Compact",
        description: "View-mode option: small dense tiles. Also its tooltip.",
    },
    "list.viewMode.list": {
        text: "List",
        description: "View-mode option: one row per operator. Also its tooltip.",
    },
    "list.stat.aria": {
        text: "Card statistic",
        description: "Accessible name of the button group choosing which community number the cards show.",
    },
    "list.stat.owned": {
        text: "Owned",
        description: "Card-statistic option: the share of players who own the operator. Uppercased by the design; keep it very short.",
    },
    "list.stat.owned.tip": {
        text: "Share of players who own each operator",
        description: "Tooltip for the Owned card statistic.",
    },
    "list.stat.e2": {
        text: "E2",
        description: "Card-statistic option: the share of owners who promoted the operator to Elite 2. The game's own shorthand, which normally stays as-is.",
    },
    "list.stat.e2.tip": {
        text: "Share of owners who promoted each operator to E2",
        description: "Tooltip for the E2 card statistic.",
    },
    "list.sort.aria": {
        text: "Sort operators",
        description: "Accessible name of the sort select.",
    },
    "list.sort.caption": {
        text: "Sort",
        description: "Small uppercase caption inside the sort control, before the chosen option. Very tight space.",
    },
    "list.sort.asc": {
        text: "Ascending",
        description: "Hover text on the sort-direction button while the order is smallest-first.",
    },
    "list.sort.desc": {
        text: "Descending",
        description: "Hover text on the sort-direction button while the order is largest-first.",
    },
    "list.sort.toggleAria": {
        text: "Toggle sort direction",
        description: "Accessible name of the arrow button that flips ascending and descending.",
    },
    "list.perPage.aria": {
        text: "Items per page",
        description: "Accessible name of the page-size select.",
    },
    "list.perPage.caption": {
        text: "Show",
        description: "Small uppercase caption inside the page-size control, before the chosen number. Very tight space.",
    },
    "list.perPage.all": {
        text: "All",
        description: "Page-size option that puts every result on one page.",
    },
    "list.export": {
        text: "Export",
        description: "Visible label on the export button, hidden on small screens.",
    },
    "list.export.tip": {
        text: "Export operators",
        description: "Accessible name and tooltip of the export button.",
    },
    "list.export.dialogTitle": {
        text: "Operators",
        description: "Title of the export dialog, naming what is being exported.",
    },
    "list.showing": {
        text: "Showing {from} to {to} of {total} operators",
        description: "Result count above the pager, e.g. 'Showing 1 to 30 of 320 operators'. All three numbers are bold and may move wherever the sentence needs them. No closing full stop.",
    },
    "list.hint": {
        text: "Hover for preview · Click to open",
        description: "Small uppercase hint beside the pager on pointer devices. Keep the middle dot separator.",
    },
    "list.empty.upcoming": {
        text: "No upcoming operators match your filters.",
        description: "Empty state when the Upcoming (CN) view is filtered down to nothing.",
    },
    "list.empty": {
        text: "No operators match your filters.",
        description: "Empty state when the grid is filtered down to nothing.",
    },
    "list.clearFilters": {
        text: "Clear all filters",
        description: "Link under the empty state that resets every filter.",
    },
    "list.column.name": {
        text: "Name",
        description: "Column header in list view.",
    },
    "list.column.rarity": {
        text: "Rarity",
        description: "Column header in list view: the star rating.",
    },
    "list.column.class": {
        text: "Class",
        description: "Column header in list view. The class values themselves are game vocabulary.",
    },
    "list.column.archetype": {
        text: "Archetype",
        description: "Column header in list view: the subclass. The values themselves are game vocabulary.",
    },
    "list.column.owned": {
        text: "Owned",
        description: "Column header in list view: the community ownership rate.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
