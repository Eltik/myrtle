import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tools";

export const messages = {
    "squad.chooser.title": {
        text: "Save to which slot?",
        description: "Title of the dialog that picks one of the player's slots to save the squad in.",
    },
    "squad.chooser.desc": {
        text: "You have {count, plural, one {# slot} other {# slots}}. Saving over a slot replaces what's there.",
        description: "Description in the save-slot chooser. {count} is how many save slots an account has.",
    },
    "squad.chooser.empty": {
        text: "Empty slot",
        description: "A slot option with nothing saved in it.",
    },
    "squad.chooser.replaces": {
        text: "Replaces {name}",
        description: "A slot option that already holds a squad. {name} is that squad's name.",
    },
    "squad.chooser.untitled": {
        text: "Squad {slot}",
        description: "Name of a saved squad that has no name. {slot} is 1 to 4.",
    },
    "squad.chooser.slot": {
        text: "Slot {slot}",
        description: "Label of a slot option, 1 to 4.",
    },
    "squad.chooser.editCopy": {
        text: "Just edit a copy",
        description: "Button in the chooser opened from a shared squad: keep it as an unsaved squad instead of saving it.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
