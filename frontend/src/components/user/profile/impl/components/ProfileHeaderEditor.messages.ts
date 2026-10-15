import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "user";

export const messages = {
    "profile.headerEditor.kicker": {
        text: "Profile header",
        description: "Small uppercase label above the editor for the numbers in the profile header and its row of stat cards.",
    },
    "profile.headerEditor.help": {
        text: "Choose the stat cards under your header, and hide any number you'd rather keep to yourself. You still see a hidden number, marked, but visitors never receive it, and it leaves the leaderboards and search sorts that rank by it.",
        description: "Explanation under the header editor's label. 'Visitors' are other people viewing this profile.",
    },
    "profile.headerEditor.cards.title": {
        text: "Stat cards",
        description: "Heading of the reorderable list of the stat cards shown under the profile header.",
    },
    "profile.headerEditor.cards.count": {
        text: "{count} of {max}",
        description: "How many stat cards are chosen out of the most allowed, beside the 'Stat cards' heading. Both are numbers.",
    },
    "profile.headerEditor.cards.empty": {
        text: "No cards: the row of stat cards is not shown.",
        description: "Shown in place of the card list when the owner removed every stat card.",
    },
    "profile.headerEditor.cards.add": {
        text: "Add a card",
        description: "Heading above the buttons that add one more stat card.",
    },
    "profile.headerEditor.cards.full": {
        text: "The row holds up to {max} cards. Remove one to add another.",
        description: "Shown under the add buttons once the stat card row is full. {max} is a number.",
    },
    "profile.headerEditor.cards.addOne": {
        text: "Add {card}",
        description: "Accessible label of a button that adds a stat card. {card} is the card's name, e.g. 'Items' or 'LMD'.",
    },
    "profile.headerEditor.cards.remove": {
        text: "Remove {card}",
        description: "Accessible label of the button that removes a stat card. {card} is the card's name.",
    },
    "profile.headerEditor.cards.moveUp": {
        text: "Move {card} earlier",
        description: "Accessible label of the button that moves a stat card one place toward the start of the row. {card} is the card's name.",
    },
    "profile.headerEditor.cards.moveDown": {
        text: "Move {card} later",
        description: "Accessible label of the button that moves a stat card one place toward the end of the row. {card} is the card's name.",
    },
    "profile.headerEditor.cards.list": {
        text: "Stat card order",
        description: "Accessible name of the reorderable list of stat cards.",
    },
    "profile.headerEditor.cards.moved": {
        text: "{card} moved to position {position} of {total}.",
        description: "Screen-reader announcement after a stat card is moved. {card} is the card's name; {position} and {total} are numbers.",
    },
    "profile.headerEditor.hidden.title": {
        text: "Hidden from visitors",
        description: "Heading of the list of header numbers, each with a toggle that hides it from other people viewing the profile.",
    },
    "profile.headerEditor.hidden.list": {
        text: "Header numbers",
        description: "Accessible name of the list of header numbers that can be hidden.",
    },
    "profile.headerEditor.hiddenBadge": {
        text: "Hidden",
        description: "Badge on a row in the header editor when that number is hidden from visitors.",
    },
    "profile.headerEditor.show": {
        text: "Show {field} to visitors",
        description: "Accessible label of the eye toggle on a hidden number. {field} is its name, e.g. 'Level'.",
    },
    "profile.headerEditor.hide": {
        text: "Hide {field} from visitors",
        description: "Accessible label of the eye toggle on a shown number. {field} is its name, e.g. 'Level'.",
    },
    "profile.headerEditor.field.level": {
        text: "Level",
        description: "The account level and its experience bar, in the list of header numbers an owner can hide.",
    },
    "profile.headerEditor.field.registered": {
        text: "Registration date",
        description: "The date the game account was created, in the list of header numbers an owner can hide.",
    },
    "profile.headerEditor.field.signIn": {
        text: "Sign-in days",
        description: "The lifetime count of days signed in, in the list of header numbers an owner can hide.",
    },
    "profile.headerEditor.reset": {
        text: "Reset to default",
        description: "Button that shows every number again and puts back the original five stat cards. Takes effect on Save.",
    },
    "profile.headerEditor.cancel": {
        text: "Cancel",
        description: "Button that closes the header editor without saving.",
    },
    "profile.headerEditor.save": {
        text: "Save",
        description: "Button that saves the stat cards and hidden numbers.",
    },
    "profile.headerEditor.saved.title": {
        text: "Header saved",
        description: "Toast title after the owner saves their header numbers and stat cards.",
    },
    "profile.headerEditor.saved.body": {
        text: "Visitors now see your header this way.",
        description: "Toast body after the owner saves their header numbers and stat cards.",
    },
    "profile.headerEditor.saveFailed.title": {
        text: "Couldn't save your header",
        description: "Toast title when saving the header numbers fails. The body explains why. Keep the apostrophe.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
