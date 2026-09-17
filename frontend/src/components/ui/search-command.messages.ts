import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

/** `ui/` primitives share one namespace: their strings are the site's chrome. */
export const namespace = "common";

export const messages = {
    "searchCommand.placeholder": {
        text: "Search operators, pages, tools…",
        description: "Placeholder in the command palette's search field. Ends in a single ellipsis character.",
    },
    "searchCommand.noResults": {
        text: "No results found.",
        description: "Shown when nothing in any group matches the query.",
    },
    "searchCommand.operators": {
        text: "Operators",
        description: "Group heading above matching operators (playable characters).",
    },
    "searchCommand.pages": {
        text: "Pages",
        description: "Group heading above matching site pages.",
    },
    "searchCommand.tools": {
        text: "Tools",
        description: "Group heading above matching tools (calculators, planners).",
    },
    "searchCommand.operatorsFailed": {
        text: "Failed to load operators. Try reopening the palette.",
        description: "Shown in place of the operator list when its request failed.",
    },
    "searchCommand.cnOnly": {
        text: "CN only",
        description: "Suffix on an operator row for an operator that exists only on the Chinese server so far. 'CN' is the server's short name and stays as-is.",
    },
    "searchCommand.noOperatorMatch": {
        text: 'No operators match "{query}".',
        description: "Shown when the operator index loaded but nothing in it matches. {query} is the text the visitor typed, quoted.",
    },
    "searchCommand.toNavigate": {
        text: "to navigate",
        description: "Footer hint, read after the up and down arrow key caps: 'up down to navigate'.",
    },
    "searchCommand.toSelect": {
        text: "to select",
        description: "Footer hint, read after the enter key cap: 'enter to select'.",
    },
    "searchCommand.poweredBy": {
        text: "powered by COSS UI",
        description: "Attribution in the palette footer. 'COSS UI' is a product name and stays as-is.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
