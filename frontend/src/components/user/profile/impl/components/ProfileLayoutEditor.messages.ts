import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "user";

export const messages = {
    "profile.layout.customize": {
        text: "Customize",
        description: "Button the profile owner sees at the end of their own tab bar. Opens the editor for tab order and visibility.",
    },
    "profile.layout.kicker": {
        text: "Profile tabs",
        description: "Small uppercase label above the tab layout editor.",
    },
    "profile.layout.help": {
        text: "Drag tabs or use the arrows to change their order. A hidden tab stays visible to you, but visitors cannot open it or load its data.",
        description: "Explanation under the tab layout editor's label. 'Visitors' are other people viewing this profile.",
    },
    "profile.layout.list": {
        text: "Tab order",
        description: "Accessible name of the reorderable list of profile tabs.",
    },
    "profile.layout.hiddenBadge": {
        text: "Hidden",
        description: "Badge on a tab row in the editor when that tab is hidden from visitors.",
    },
    "profile.layout.show": {
        text: "Show {tab} to visitors",
        description: "Accessible label of the eye toggle on a hidden tab. {tab} is the tab's name, e.g. 'Roster'.",
    },
    "profile.layout.hide": {
        text: "Hide {tab} from visitors",
        description: "Accessible label of the eye toggle on a visible tab. {tab} is the tab's name, e.g. 'Roster'.",
    },
    "profile.layout.moveUp": {
        text: "Move {tab} earlier",
        description: "Accessible label of the button that moves a tab one place toward the start of the tab bar. {tab} is the tab's name.",
    },
    "profile.layout.moveDown": {
        text: "Move {tab} later",
        description: "Accessible label of the button that moves a tab one place toward the end of the tab bar. {tab} is the tab's name.",
    },
    "profile.layout.moved": {
        text: "{tab} moved to position {position} of {total}.",
        description: "Screen-reader announcement after a tab is moved. {tab} is the tab's name; {position} and {total} are numbers.",
    },
    "profile.layout.reset": {
        text: "Reset to default",
        description: "Button that puts every tab back in its original order and makes all of them visible. Takes effect on Save.",
    },
    "profile.layout.cancel": {
        text: "Cancel",
        description: "Button that closes the tab editor without saving.",
    },
    "profile.layout.save": {
        text: "Save",
        description: "Button that saves the tab order and visibility.",
    },
    "profile.layout.allHidden": {
        text: "Every tab is hidden, so visitors will see only your profile header.",
        description: "Warning in the tab editor when the owner has hidden every tab.",
    },
    "profile.layout.saved.title": {
        text: "Tabs saved",
        description: "Toast title after the owner saves their tab layout.",
    },
    "profile.layout.saved.body": {
        text: "Visitors now see your tabs in this order.",
        description: "Toast body after the owner saves their tab layout.",
    },
    "profile.layout.saveFailed.title": {
        text: "Couldn't save your tabs",
        description: "Toast title when saving the tab layout fails. The body explains why.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
