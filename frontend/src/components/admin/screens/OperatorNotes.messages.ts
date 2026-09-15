import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "admin";

export const messages = {
    "notes.kicker": {
        text: "Manage",
        description: "Eyebrow over the Operator notes page title, naming the admin section it belongs to. Rendered uppercase.",
    },
    "notes.title": {
        text: "Operator notes",
        description: "Title of the admin screen that edits the guidance attached to each operator.",
    },
    "notes.sub": {
        text: "Community guidance attached to each operator. Edits write to {table} and append a diff to {auditTable}.",
        description: "Sentence under the Operator notes title. {table} and {auditTable} are database table names, shown in monospace. 'Operator' is the game's word for a playable character; a 'diff' is a record of what changed.",
    },
    "notes.searchPlaceholder": {
        text: "Search name, char_id, summary, or tag…",
        description: "Prompt in the notes search box. 'char_id' is the game's internal operator identifier and stays as-is. Keep the ellipsis character.",
    },
    "notes.filter.all": {
        text: "All · {count}",
        description: "Filter tab showing every operator note, with how many there are. Keep the middle dot.",
    },
    "notes.filter.hasContent": {
        text: "Has content · {count}",
        description: "Filter tab showing only notes that have been written, with how many there are. Keep the middle dot.",
    },
    "notes.filter.empty": {
        text: "Empty · {count}",
        description: "Filter tab showing only notes that are still blank, with how many there are. Keep the middle dot.",
    },
    "notes.sort.recent": {
        text: "Recently updated",
        description: "Sort option: newest edits first.",
    },
    "notes.sort.name": {
        text: "Name (A→Z)",
        description: "Sort option: alphabetical by operator name. Keep the arrow.",
    },
    "notes.sort.rarity": {
        text: "Rarity (high→low)",
        description: "Sort option: highest-rarity operators first. 'Rarity' is the game's star rating. Keep the arrow.",
    },
    "notes.countShown": {
        text: "{filtered} of {total}",
        description: "How many operator notes survive the current filter.",
    },
    "notes.noMatch": {
        text: "No notes match these filters.",
        description: "Empty state when notes exist but none survive the current filter. A 'Reset' button follows it.",
    },
    "notes.reset": {
        text: "Reset",
        description: "Button that clears the note search and filter.",
    },
    "notes.empty": {
        text: "No operator notes yet.",
        description: "Empty state when no operator has a note at all.",
    },
    "notes.noSummary": {
        text: "No summary yet - click to add one.",
        description: "Placeholder line on a note row with no summary written. Keep the hyphen.",
    },
    "notes.badge.filled": {
        text: "filled",
        description: "Badge on a note row that has content.",
    },
    "notes.badge.empty": {
        text: "empty",
        description: "Badge on a note row that is still blank.",
    },
    "notes.operatorFallback": {
        text: "Operator",
        description: "Stand-in heading when the operator's name has not loaded. 'Operator' is the game's word for a playable character.",
    },
    "notes.editor.lastUpdated": {
        text: "Last updated {when} · {count, plural, one {# revision} other {# revisions}}",
        description: "Caption over the note editor; {when} is an already-formatted relative time such as '3 days ago'. Keep the middle dot.",
    },
    "notes.editor.new": {
        text: "New note - nothing saved yet.",
        description: "Caption over the note editor for an operator with no saved note. Keep the hyphen.",
    },
    "notes.back": {
        text: "Back",
        description: "Button that leaves the note editor for the note list.",
    },
    "notes.save": {
        text: "Save",
        description: "Button that writes the note being edited.",
    },
    "notes.field.summary": {
        text: "Summary",
        description: "Label of the one-line summary field, and the heading over it in the preview.",
    },
    "notes.field.summary.hint": {
        text: "One-line synopsis for the operator card.",
        description: "Helper text under the summary field, naming where the summary is shown.",
    },
    "notes.field.pros": {
        text: "Pros",
        description: "Label of the strengths field, and the heading over it in the preview.",
    },
    "notes.field.pros.placeholder": {
        text: "What's strong about this operator?",
        description: "Prompt inside the empty strengths field. 'Operator' is the game's word for a playable character.",
    },
    "notes.field.cons": {
        text: "Cons",
        description: "Label of the weaknesses field, and the heading over it in the preview.",
    },
    "notes.field.cons.placeholder": {
        text: "Where do they fall short?",
        description: "Prompt inside the empty weaknesses field.",
    },
    "notes.field.notes": {
        text: "Notes",
        description: "Label of the long-form guidance field, and the heading over it in the preview.",
    },
    "notes.field.notes.placeholder": {
        text: "Deeper guidance, usage tips, synergies.",
        description: "Prompt inside the empty guidance field. A 'synergy' is a pairing that works well together.",
    },
    "notes.field.trivia": {
        text: "Trivia",
        description: "Label of the trivia field, and the heading over it in the preview.",
    },
    "notes.field.trivia.placeholder": {
        text: "Lore, fun facts.",
        description: "Prompt inside the empty trivia field. 'Lore' is the game's own story background.",
    },
    "notes.field.tags": {
        text: "Tags",
        description: "Label of the field that holds the note's free-form tags.",
    },
    "notes.tagPlaceholder": {
        text: "Add a tag…",
        description: "Prompt inside the empty tag input. Keep the ellipsis character.",
    },
    "notes.addTag": {
        text: "Add",
        description: "Button that commits the typed tag. A verb.",
    },
    "notes.removeTag": {
        text: "Remove {tag}",
        description: "Accessible name of the X button on a tag chip; {tag} is the tag's own text.",
    },
    "notes.preview": {
        text: "Preview",
        description: "Heading over the rendered version of the note being edited. Rendered uppercase.",
    },
    "notes.unsaved": {
        text: "unsaved changes",
        description: "Badge over the preview when the editor holds edits that have not been written yet.",
    },
    "notes.saved": {
        text: "saved",
        description: "Badge over the preview when the editor matches what is stored.",
    },
    "notes.preview.empty": {
        text: "Nothing yet - start writing on the left.",
        description: "Empty state in the note preview. 'On the left' is where the edit fields are. Keep the hyphen.",
    },
    "notes.history.title": {
        text: "Revision history",
        description: "Heading of the card listing past edits to this note.",
    },
    "notes.history.desc": {
        text: "Each save appends a row to {table} with a JWT-signed actor stamp.",
        description: "Caption under the Revision history heading: each row records who made the edit, signed by the sign-in token. {table} is a database table name, shown in monospace. 'JWT' is the token format and stays as-is.",
    },
    "notes.history.empty": {
        text: "No revisions yet.",
        description: "Empty state in the Revision history card.",
    },
    "notes.uid": {
        text: "UID",
        description: "Label before an account number in the revision list. 'UID' is the game's own abbreviation and stays as-is.",
    },
    "notes.history.updated": {
        text: "{actor} updated {field}",
        description: "One revision row in the note's history. {actor} is the editor's account id in monospace and {field} the note field they changed.",
    },
    "notes.toast.saved": {
        text: "Note saved",
        description: "Toast title after a note was written.",
    },
    "notes.toast.saved.desc": {
        text: "Updated operator notes for {name}.",
        description: "Toast body after a note was written; {name} is the operator's name, which comes from game data.",
    },
    "notes.toast.failed": {
        text: "Failed to save",
        description: "Toast title after a note write was rejected.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
