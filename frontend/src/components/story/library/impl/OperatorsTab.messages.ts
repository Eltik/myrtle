import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "story";

export const messages = {
    "operators.search.placeholder": {
        text: "Search operators",
        description: "Placeholder of the search box above the operator records grid.",
    },
    "operators.search.aria": {
        text: "Search operators",
        description: "Accessible name of the search box above the operator records grid.",
    },
    "operators.hideFinished": {
        text: "Hide finished",
        description: "Checkbox that removes operators whose records are all read from the grid.",
    },
    "operators.readCount": {
        text: "{done} of {total} operators read through",
        description: "Counter above the operator records grid.",
    },
    "operators.empty": {
        text: "No operator matches.",
        description: "Empty state under the operator records grid when the search and filters leave nothing.",
    },
    "operators.showMore": {
        text: "Show {count} more ({remaining} left)",
        description: "Button under the operator records grid in the browse page that mounts the next page of cards. count is how many the press adds, remaining how many are still hidden.",
    },
    "operators.card.read": {
        text: "{read}/{total}",
        description: "Read fraction on an operator card: records read over records that have a script.",
    },
    "operators.card.open": {
        text: "Show {name}'s records",
        description: "Accessible name of an operator card that opens its record dialog.",
    },
    "operators.dialog.close": {
        text: "Close",
        description: "Button that closes the operator record dialog.",
    },
    "operators.rarity": {
        text: "{count, plural, one {# star} other {# stars}}",
        description: "Accessible name of the rarity stars on an operator card.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
