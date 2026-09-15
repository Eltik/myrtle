import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "stages";

export const messages = {
    "list.breadcrumb": {
        text: "Breadcrumb",
        description: "Accessible name of the breadcrumb <nav> landmark at the top of the stage list.",
    },
    "list.breadcrumb.collection": {
        text: "Collection",
        description: "First breadcrumb crumb, naming the section the stage list lives in.",
    },
    "list.breadcrumb.stages": {
        text: "Stages",
        description: "Last breadcrumb crumb: this page.",
    },
    "list.kicker": {
        text: "The Stage Record",
        description: "Small uppercase kicker over the page heading. This site's own name for the stage archive.",
    },
    "list.title": {
        text: "Every operation, catalogued.",
        description: "Page heading of the stage list. 'Operation' is the game's word for a stage.",
    },
    "list.blurb": {
        text: "Browse by story arc and code - each stage opens to a faux-3D board and enemy-pathing simulator. Ordered by episode and code.",
        description: "Paragraph under the stage-list heading, explaining what the list offers.",
    },
    "list.counts": {
        text: "STAGES · {zones} ZONES · {categories} CATEGORIES",
        description: "Metric line beside the total stage count, which is rendered separately in large type just above it. Uppercase on purpose. {zones} and {categories} are counts.",
    },
    "list.search.placeholder": {
        text: "Search event, code or stage…",
        description: "Placeholder in the stage-list search box. Ends with a single ellipsis character.",
    },
    "list.search.aria": {
        text: "Search stages",
        description: "Accessible name of the stage-list search box.",
    },
    "list.count.matches": {
        text: "{count, plural, one {{count} match} other {{count} matches}}",
        description: "How many stages the current search matches, beside the search box.",
    },
    "list.count.zones": {
        text: "{count, plural, one {{count} zone} other {{count} zones}}",
        description: "How many zones are listed when no search is active, beside the search box.",
    },
    "list.collapse": {
        text: "Collapse",
        description: "Button that folds every zone row in the list shut.",
    },
    "list.expandAll": {
        text: "Expand all",
        description: "Button that opens every zone row in the list.",
    },
    "list.filter.all": {
        text: "All",
        description: "First category filter pill: no category filter. The other pills are named after the game's own modes and come from the registry.",
    },
    "list.empty": {
        text: "No stages match your search.",
        description: "Empty state in place of the stage list when the search and filter match nothing.",
    },
    "list.group.counts": {
        text: "{zones} zones · {stages} stages",
        description: "Counts beside a category heading. Both are numbers; the category name itself is the game's own mode name.",
    },
    "list.zone.collapse": {
        text: "Collapse {title}",
        description: "Accessible name of an open zone row's toggle. {title} is the zone/event name from the game data.",
    },
    "list.zone.expand": {
        text: "Expand {title}",
        description: "Accessible name of a closed zone row's toggle. {title} is the zone/event name from the game data.",
    },
    "list.zone.about": {
        text: "About {title}",
        description: "Accessible name of the zone thumbnail, which opens the enlarged-banner detail dialog. {title} is the zone/event name from the game data.",
    },
    "list.zone.ops": {
        text: "{count, plural, one {{count} op} other {{count} ops}}",
        description: "How many operations a zone row holds. 'op' is short for 'operation'; the row is narrow, so keep it short.",
    },
    "list.zone.boss": {
        text: "{count} boss",
        description: "How many boss operations a zone row holds, after a '·' separator. Deliberately not inflected: it reads as a terse metric label, not a sentence.",
    },
    "list.card.boss": {
        text: "Boss",
        description: "Badge in the corner of a stage card marking an operation with a boss enemy. Very tight space.",
    },
    "list.card.sanity": {
        text: "{cost} ◆ Sanity",
        description: "Entry cost on a stage card. 'Sanity' is the game's own stamina currency and '◆' is its icon; {cost} is the number.",
    },
    "list.card.free": {
        text: "Free entry",
        description: "Shown on a stage card in place of the Sanity cost when the operation costs nothing to enter.",
    },
    "list.card.cm": {
        text: "CM",
        description: "Marker on a stage card meaning a Challenge Mode version of the operation also exists. Two characters at most.",
    },
    "list.hero.operations": {
        text: "{count, plural, one {operation} other {operations}}",
        description: "Unit after the operation count on the featured banner; the number is rendered just before it and is not part of this string.",
    },
    "list.hero.bosses": {
        text: "{count, plural, one {boss} other {bosses}}",
        description: "Unit after the boss count on the featured banner; the number is rendered just before it and is not part of this string.",
    },
    "list.hero.browse": {
        text: "Browse stages",
        description: "Primary button on the featured banner: scroll to and open that event's row in the list.",
    },
    "list.hero.details": {
        text: "Details",
        description: "Secondary button on the featured banner: open the enlarged-banner detail dialog.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
