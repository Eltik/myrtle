import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tools";

export const messages = {
    "squad.picker.title": {
        text: "Add to seat {seat}",
        description: "Heading of the operator picker that opens under the grid when an empty seat is clicked. {seat} is 1 to 12.",
    },
    "squad.picker.supportTitle": {
        text: "Pick a support",
        description: "Heading of the operator picker for the support seat (an operator borrowed from a friend).",
    },
    "squad.picker.close": {
        text: "Close",
        description: "Button that closes the operator picker.",
    },
    "squad.picker.search": {
        text: "Search operators",
        description: "Placeholder and accessible name of the search box in the operator picker.",
    },
    "squad.picker.owned": {
        text: "Owned only",
        description: "Switch in the operator picker that hides operators the signed-in player does not have.",
    },
    "squad.picker.class": {
        text: "Class",
        description: "Accessible name of the class filter in the operator picker.",
    },
    "squad.picker.rarity": {
        text: "Rarity",
        description: "Accessible name of the rarity filter in the operator picker.",
    },
    "squad.picker.all": {
        text: "All",
        description: "Filter option that shows every class or every rarity.",
    },
    "squad.picker.stars": {
        text: "{rarity}★",
        description: "A rarity filter option, e.g. '6★'.",
    },
    "squad.picker.seated": {
        text: "Seat {seat}",
        description: "Badge on an operator tile that is already in the squad. {seat} is the seat number, 1 to 12.",
    },
    "squad.picker.seatedSupport": {
        text: "Support",
        description: "Badge on an operator tile that is already the support.",
    },
    "squad.picker.level": {
        text: "Lv {level}",
        description: "Roster level under an owned operator's tile, e.g. 'Lv 90'.",
    },
    "squad.picker.pick": {
        text: "Add {name}",
        description: "Accessible name of an operator tile in the picker. {name} is the operator's name (game text).",
    },
    "squad.picker.count": {
        text: "{count, plural, one {# operator} other {# operators}}",
        description: "Result count above the operator picker grid.",
    },
    "squad.picker.empty": {
        text: "No operators match.",
        description: "Shown in the picker when the search and filters leave nothing.",
    },
    "squad.picker.loading": {
        text: "Loading operators...",
        description: "Shown in the picker while the operator list loads.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
