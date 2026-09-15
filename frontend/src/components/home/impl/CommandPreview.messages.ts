import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "home";

export const messages = {
    "cmd.open": {
        text: "Open command palette",
        description: "Accessible name of the fake search field in the landing-page command-palette mockup; clicking it opens the real palette.",
    },
    "cmd.searchIcon": {
        text: "Search",
        description: "Accessible name of the magnifier icon in the command-palette mockup.",
    },
    "cmd.group.operators": {
        text: "Operators",
        description: "Group heading above the sample operators (playable characters) in the command-palette mockup.",
    },
    "cmd.group.tools": {
        text: "Tools",
        description: "Group heading above the sample tools (calculators, planners) in the command-palette mockup.",
    },
    "cmd.toNavigate": {
        text: "to navigate",
        description: "Footer hint in the command-palette mockup, read after the up and down arrow key caps: 'up down to navigate'.",
    },
    "cmd.toSelect": {
        text: "to select",
        description: "Footer hint in the command-palette mockup, read after the enter key cap: 'enter to select'.",
    },
    "cmd.poweredBy": {
        text: "powered by COSS UI",
        description: "Attribution in the command-palette mockup footer. 'COSS UI' is a product name and stays as-is.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
