import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "story";

export const messages = {
    "operators.dialog.tab.records": {
        text: "Records",
        description: "Tab in the operator dialog holding the operator's story records.",
    },
    "operators.dialog.tab.files": {
        text: "Files",
        description: "Tab in the operator dialog holding the handbook archive sections.",
    },
    "operators.dialog.tab.modules": {
        text: "Modules",
        description: "Tab in the operator dialog holding the module texts.",
    },
    "operators.dialog.tab.voices": {
        text: "Voice Lines",
        description: "Tab in the operator dialog holding the operator's voice lines.",
    },
    "operators.dialog.words": {
        text: "{count} words",
        description: "Word count beside a section title or a tab's summary line.",
    },
    "operators.dialog.shelfWords": {
        text: "{count} rows · {words} words",
        description: "Summary line above a shelf in the operator dialog: how many sections and how many words they hold.",
    },
    "operators.dialog.loading": {
        text: "Loading…",
        description: "Placeholder while a shelf's query is in flight in the operator dialog.",
    },
    "operators.dialog.failed": {
        text: "This shelf did not load. Close the dialog and open it again.",
        description: "Message shown in the operator dialog when a shelf's query failed.",
    },
    "operators.dialog.files.empty": {
        text: "No archive files for this operator.",
        description: "Empty state of the Files tab in the operator dialog.",
    },
    "operators.dialog.modules.empty": {
        text: "Not available yet.",
        description: "Empty state of the Modules tab in the operator dialog when the operator has no module text on the wire.",
    },
    "operators.dialog.modules.emptyNote": {
        text: "This operator's modules carry no story text on the wire, or they have none yet.",
        description: "Explanation under the Modules tab's empty state in the operator dialog.",
    },
    "operators.dialog.voices.empty": {
        text: "No voice lines for this operator.",
        description: "Empty state of the Voice Lines tab in the operator dialog.",
    },
    "operators.dialog.section.expand": {
        text: "Show {name}",
        description: "Accessible name of the control that expands one section in the operator dialog.",
    },
    "operators.dialog.section.collapse": {
        text: "Hide {name}",
        description: "Accessible name of the control that collapses one section in the operator dialog.",
    },
    "operators.dialog.original": {
        text: "Original",
        description: "Badge on the operator's starting kit in the Modules tab, as opposed to an unlockable module.",
    },
    "operators.dialog.voices.language": {
        text: "Language",
        description: "Accessible name of the language select above the voice lines.",
    },
    "operators.dialog.voices.play": {
        text: "Play {name}",
        description: "Accessible name of the play button on a voice line.",
    },
    "operators.dialog.voices.pause": {
        text: "Pause {name}",
        description: "Accessible name of the play button on a voice line while it plays.",
    },
    "operators.dialog.voices.noClip": {
        text: "No clip in this language",
        description: "Accessible name of the disabled play button on a voice line with no audio in the chosen language.",
    },
    "operators.dialog.lazyNote": {
        text: "Files, Modules and Voice Lines each load when you open them, so their counts are in here rather than on the card.",
        description: "Muted note in the operator dialog saying why the three later shelves carry no count on the operator card.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
