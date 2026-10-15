import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tools";

export const messages = {
    "squad.breadcrumb.tools": {
        text: "Tools",
        description: "First crumb of the breadcrumb trail, naming the section this tool lives in.",
    },
    "squad.title": {
        text: "Squad Builder",
        description: "Page heading and last breadcrumb of the Squad Builder.",
    },
    "squad.intro": {
        text: "Put together {seats} operators and a support, then see what the squad covers, what it costs to deploy and how it works together.",
        description: "One-line description under the Squad Builder heading. {seats} is the seats in a squad (12).",
    },
    "squad.breadcrumbLabel": {
        text: "Breadcrumb",
        description: "Accessible name of the breadcrumb navigation.",
    },
    "squad.shared.title": {
        text: "Shared squad",
        description: "Banner title when the page opened a squad from a share link.",
    },
    "squad.shared.desc": {
        text: "Someone sent you this squad. It's read only until you copy it.",
        description: "Banner text for a squad opened from a share link.",
    },
    "squad.shared.copy": {
        text: "Copy to a slot",
        description: "Banner button that saves the shared squad into one of the player's slots.",
    },
    "squad.shared.editCopy": {
        text: "Edit a copy",
        description: "Banner button (signed out) that turns the shared squad into the player's own unsaved squad.",
    },
    "squad.shared.close": {
        text: "Back to my squad",
        description: "Banner button that leaves the shared squad and shows the player's own squad again.",
    },
    "squad.leave.title": {
        text: "Leave without saving?",
        description: "Title of the confirm dialog when loading another squad over unsaved changes.",
    },
    "squad.leave.desc": {
        text: "Your changes to this squad aren't saved yet.",
        description: "Body of the leave-without-saving dialog.",
    },
    "squad.leave.keep": {
        text: "Keep editing",
        description: "Button that cancels loading another squad.",
    },
    "squad.leave.discard": {
        text: "Discard",
        description: "Button that throws away the unsaved changes and loads the other squad.",
    },
    "squad.toast.saved": {
        text: "Saved to slot {slot}",
        description: "Toast after the squad was saved.",
    },
    "squad.toast.saveFailed": {
        text: "Couldn't save the squad",
        description: "Toast title when saving failed; the reason follows.",
    },
    "squad.toast.deleted": {
        text: "Slot {slot} deleted",
        description: "Toast after a saved squad was deleted.",
    },
    "squad.toast.deleteFailed": {
        text: "Couldn't delete the slot",
        description: "Toast title when deleting failed; the reason follows.",
    },
    "squad.toast.stageFailed": {
        text: "Couldn't load that stage",
        description: "Toast title when the picked stage failed to load.",
    },
    "squad.defaultName": {
        text: "Squad {slot}",
        description: "Name given to a squad saved without a name. {slot} is 1 to 4.",
    },
    "squad.importedName": {
        text: "{name} (copy)",
        description: "Name given to a copy of an in-game squad loaded into the builder. {name} is the in-game squad's name.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
