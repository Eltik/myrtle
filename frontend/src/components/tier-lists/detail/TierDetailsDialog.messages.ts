import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tierLists";

export const messages = {
    "detail.tierDialog.kicker": {
        text: "Tier",
        description: "Small uppercase label above the tier's name in its details dialog.",
    },
    "detail.tierDialog.opCount": {
        text: "{count, plural, one {op} other {ops}}",
        description: "Follows the operator count in the dialog header; the number itself is rendered just before it. Abbreviated from 'operator' because the line is tight.",
    },
    "detail.tierDialog.updated": {
        text: "Updated {when}",
        description: "In the dialog header: how long ago an operator in this tier last changed, e.g. 'Updated 3 days ago'. Rendered uppercase.",
    },
    "detail.tierDialog.noDescription": {
        text: "The author hasn't written a description for this tier.",
        description: "Stand-in in the dialog body when the author wrote no description for the tier.",
    },
    "detail.tierDialog.overviewLabel": {
        text: "Tier overview",
        description: "Accessible name of the three summary tiles at the top of the dialog.",
    },
    "detail.tierDialog.stat.operators": {
        text: "Entries",
        description: "Summary tile label: how many entries (operators, enemies, events or any other ranked thing) sit in this tier. Rendered uppercase.",
    },
    "detail.tierDialog.stat.avgRarity": {
        text: "Avg ★",
        description: "Summary tile label: the mean star rating of the tier's operators. 'Avg' is abbreviated because the tile is narrow.",
    },
    "detail.tierDialog.stat.meleeRanged": {
        text: "Melee / Ranged",
        description: "Summary tile label over a 'melee count / ranged count' pair. Melee and Ranged are the game's own deployment positions.",
    },
    "detail.tierDialog.rarityLabel": {
        text: "Rarity breakdown",
        description: "Accessible name of the section counting the tier's operators by star rating.",
    },
    "detail.tierDialog.rarityHeading": {
        text: "Rarity",
        description: "Heading of the section counting the tier's operators by star rating. Rendered uppercase.",
    },
    "detail.tierDialog.classLabel": {
        text: "Class breakdown",
        description: "Accessible name of the section counting the tier's operators by class.",
    },
    "detail.tierDialog.classHeading": {
        text: "Classes",
        description: "Heading of the section counting the tier's operators by class. Rendered uppercase.",
    },
    "detail.tierDialog.operatorsLabel": {
        text: "Entries in this tier",
        description: "Accessible name of the list of the tier's entries.",
    },
    "detail.tierDialog.operatorsHeading": {
        text: "Entries",
        description: "Heading of the list of the tier's entries (operators, enemies, events or any other ranked thing). Rendered uppercase.",
    },
    "detail.tierDialog.operatorsEmpty": {
        text: "Nothing has been placed in this tier yet.",
        description: "Empty state in place of the list of entries when the tier holds none.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
