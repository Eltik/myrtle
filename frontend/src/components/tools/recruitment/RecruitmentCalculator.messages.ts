import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tools";

export const messages = {
    "recruit.breadcrumb.tools": {
        text: "Tools",
        description: "First crumb of the breadcrumb trail, naming the section this tool lives in.",
    },
    "recruit.title": {
        text: "Recruitment Calculator",
        description: "Page heading and last breadcrumb. 'Recruitment' is the in-game screen's name.",
    },
    "recruit.intro": {
        text: 'Pick the tags shown in your recruitment screen - up to {max}. Combinations are ranked by guaranteed minimum rarity - the best worst case first. Six-star operators only appear in combinations that include "Top Operator".',
        description: "Blurb under the page heading. 'Top Operator' is a recruitment tag name from the game data, so use whatever the game calls that tag in this region, in the game's own quotes. The dashes are plain hyphens.",
    },
    "recruit.tags": {
        text: "Tags",
        description: "Heading of the card holding the recruitment-tag buttons.",
    },
    "recruit.tags.count": {
        text: "({selected}/{max})",
        description: "How many tags are picked out of the maximum, e.g. '(3/5)'. Keep the parentheses.",
    },
    "recruit.options": {
        text: "Options",
        description: "Heading of the card holding the which-operators-count switches and the sort order.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
