import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "admin";

export const messages = {
    "notes.head.kicker": {
        text: "Manage",
        description: "Small label above the Operator notes page title: the admin area this page belongs to.",
    },
    "notes.head.title": {
        text: "Operator notes",
        description: "Title of the admin page for editing the guidance shown on each operator's page.",
    },
    "notes.head.sub": {
        text: "Guidance shown on each operator's page. Use Save & next to work through the ones not done yet.",
        description: "Line under the Operator notes title. 'Save & next' is the editor button that saves and opens the next operator whose note is empty or incomplete.",
    },

    "notes.list.search": {
        text: "Search operators",
        description: "Placeholder and accessible name of the search box above the operator list on the Operator notes page. Matches names, ids and tags.",
    },
    "notes.status.all": {
        text: "All",
        description: "Status filter chip on the Operator notes list: show every operator. A count follows it.",
    },
    "notes.status.empty": {
        text: "Empty",
        description: "Status for an operator with no notes written yet. Used as a filter chip (with a count), as the badge on a list row and as the removable active-filter chip.",
    },
    "notes.status.incomplete": {
        text: "Incomplete",
        description: "Status for an operator whose note has some fields written but not all five (summary, pros, cons, notes, trivia). Used as a filter chip (with a count), as the badge on a list row and as the removable active-filter chip.",
    },
    "notes.status.filled": {
        text: "Done",
        description: "Status for an operator whose note has all five fields written (summary, pros, cons, notes, trivia). Used as a filter chip (with a count), as the badge on a list row and as the removable active-filter chip.",
    },
    "notes.filters.rarity": {
        text: "Rarity",
        description: "Heading of the rarity group in the Operator notes filter panel; star-count chips follow it.",
    },
    "notes.filters.rarityChip": {
        text: "{rarity}★",
        description: "An operator rarity as a star count, e.g. '6★'. Used as a filter chip and in the operator's details line. {rarity} is a number 1-6.",
    },
    "notes.filters.tags": {
        text: "Tags",
        description: "Heading of the tags group in the Operator notes filter panel; one chip per tag used on any note.",
    },
    "notes.filters.noTags": {
        text: "No tags in use yet.",
        description: "Shown in the Tags group of the filter panel when no operator note has a tag.",
    },
    "notes.filters.sort": {
        text: "Sort by",
        description: "Heading of the sort-order group in the Operator notes filter panel.",
    },
    "notes.sortBy.name": {
        text: "Name",
        description: "Sort option in the Operator notes filter panel: alphabetical by operator name (the default).",
    },
    "notes.sortBy.rarity": {
        text: "Rarity",
        description: "Sort option in the Operator notes filter panel: highest rarity first.",
    },
    "notes.sortBy.recent": {
        text: "Recently updated",
        description: "Sort option in the Operator notes filter panel: most recently edited notes first. Also the removable chip shown while this sort is on.",
    },
    "notes.chips.query": {
        text: "“{q}”",
        description: "Removable active-filter chip showing the current search text in quotes. {q} is what the user typed.",
    },
    "notes.chips.sortedRarity": {
        text: "Sorted by rarity",
        description: "Removable active-filter chip shown while the Operator notes list is sorted by rarity.",
    },
    "notes.list.meta": {
        text: "{rarity}★",
        description: "Details line under an operator's name in the Operator notes list when the note has no tags. {rarity} is the star count.",
    },
    "notes.list.metaTags": {
        text: "{rarity}★ · {tags}",
        description: "Details line under an operator's name in the Operator notes list. {rarity} is the star count, {tags} the note's tags joined by commas.",
    },
    "notes.list.noMatch": {
        text: "No operators match.",
        description: "Shown in the Operator notes list when the search and filters leave no operator.",
    },
    "notes.list.loadFailed": {
        text: "Couldn't load the operators.",
        description: "Shown in the Operator notes list when the operator list or the notes failed to load.",
    },
    "notes.list.retry": {
        text: "Try again",
        description: "Button that reloads the operator list after it failed to load.",
    },

    "notes.detail.back": {
        text: "← All operators",
        description: "Phone-only button above the note editor that goes back to the operator list.",
    },
    "notes.detail.meta": {
        text: "{rarity}★ · {id}",
        description: "Details line under the operator's name in the note editor, for an operator with no saved note. {rarity} is the star count, {id} the operator's internal id.",
    },
    "notes.detail.metaUpdated": {
        text: "{rarity}★ · {id} · last updated {when}",
        description: "Details line under the operator's name in the note editor. {rarity} is the star count, {id} the operator's internal id, {when} a short relative time such as '3d ago'.",
    },
    "notes.detail.none": {
        text: "Pick an operator to edit their notes.",
        description: "Shown in the editor panel when no operator is selected.",
    },
    "notes.detail.missing": {
        text: "Missing: {fields}",
        description: "Quiet line under the operator's name in the note editor when the note is only partly written. {fields} lists the blank fields, e.g. 'Pros, Cons'.",
    },
    "notes.detail.unsaved": {
        text: "Unsaved changes",
        description: "Warning next to the Save buttons when the note has edits that are not saved yet.",
    },
    "notes.detail.reset": {
        text: "Reset",
        description: "Button that throws away the unsaved edits to this operator's note.",
    },
    "notes.detail.save": {
        text: "Save",
        description: "Button that saves this operator's note.",
    },
    "notes.detail.saveNext": {
        text: "Save & next",
        description: "Button that saves this operator's note and opens the next operator in the list whose note is empty or incomplete.",
    },
    "notes.mode.label": {
        text: "Editor view",
        description: "Accessible name of the Write / Preview switch above the note fields.",
    },
    "notes.mode.write": {
        text: "Write",
        description: "Tab that shows the editable note fields.",
    },
    "notes.mode.preview": {
        text: "Preview",
        description: "Tab that shows the note rendered as Markdown, the way the operator's page shows it.",
    },
    "notes.previewPane.nothing": {
        text: "Nothing written yet.",
        description: "Shown in the note preview when every field is empty.",
    },

    "notes.form.summary": {
        text: "Summary · shown on the operator card",
        description: "Label of the one-line summary field in the note editor; the summary appears on the operator's card.",
    },
    "notes.form.summaryCount": {
        text: "{count} / {limit}",
        description: "Character counter next to the summary field, e.g. '120 / 280'. Turns red past the limit.",
    },
    "notes.form.summaryPlaceholder": {
        text: "One line on what this operator is for",
        description: "Placeholder of the summary field in the note editor.",
    },
    "notes.form.pros": {
        text: "Pros",
        description: "Label of the field listing an operator's strengths. Also a heading in the note preview.",
    },
    "notes.form.prosPlaceholder": {
        text: "What's strong about this operator?",
        description: "Placeholder of the Pros field in the note editor.",
    },
    "notes.form.cons": {
        text: "Cons",
        description: "Label of the field listing an operator's weaknesses. Also a heading in the note preview.",
    },
    "notes.form.consPlaceholder": {
        text: "Where do they fall short?",
        description: "Placeholder of the Cons field in the note editor.",
    },
    "notes.form.notes": {
        text: "Notes",
        description: "Label of the free-form guidance field in the note editor. Also a heading in the note preview.",
    },
    "notes.form.notesPlaceholder": {
        text: "Deeper guidance, usage tips, synergies.",
        description: "Placeholder of the Notes field in the note editor.",
    },
    "notes.form.trivia": {
        text: "Trivia",
        description: "Label of the lore / fun facts field in the note editor. Also a heading in the note preview.",
    },
    "notes.form.triviaPlaceholder": {
        text: "Lore, fun facts.",
        description: "Placeholder of the Trivia field in the note editor.",
    },
    "notes.form.tags": {
        text: "Tags",
        description: "Label of the tag list in the note editor.",
    },
    "notes.form.tagPlaceholder": {
        text: "Add tag, press Enter",
        description: "Placeholder of the small input that adds a tag to the note when Enter is pressed.",
    },
    "notes.form.removeTag": {
        text: "Remove tag",
        description: "Tooltip on a tag pill in the note editor; clicking the pill removes the tag.",
    },
    "notes.form.removeTagNamed": {
        text: "Remove tag {tag}",
        description: "Accessible name of a tag pill in the note editor. {tag} is the tag text.",
    },

    "notes.fieldName.summary": {
        text: "Summary",
        description: "Name of the summary field, shown as a badge on a revision-history row.",
    },
    "notes.fieldName.pros": {
        text: "Pros",
        description: "Name of the Pros field, shown as a badge on a revision-history row.",
    },
    "notes.fieldName.cons": {
        text: "Cons",
        description: "Name of the Cons field, shown as a badge on a revision-history row.",
    },
    "notes.fieldName.notes": {
        text: "Notes",
        description: "Name of the Notes field, shown as a badge on a revision-history row.",
    },
    "notes.fieldName.trivia": {
        text: "Trivia",
        description: "Name of the Trivia field, shown as a badge on a revision-history row.",
    },
    "notes.fieldName.tags": {
        text: "Tags",
        description: "Name of the tags field, shown as a badge on a revision-history row.",
    },

    "notes.revisions.title": {
        text: "Revision history",
        description: "Heading of the list of past edits to this operator's note, when there are none.",
    },
    "notes.revisions.titleCount": {
        text: "Revision history · {count, plural, one {# change} other {# changes}}",
        description: "Heading of the list of past edits to this operator's note, with how many edits were recorded.",
    },
    "notes.revisions.empty": {
        text: "Nothing saved for this operator yet.",
        description: "Shown under Revision history when the note was never saved.",
    },
    "notes.revisions.loadFailed": {
        text: "Couldn't load the revision history.",
        description: "Shown under Revision history when the edit log failed to load.",
    },
    "notes.revisions.added": {
        text: "added",
        description: "Verb on a revision row: the editor (named before it) filled in a field that was empty. The field's name follows as a badge.",
    },
    "notes.revisions.cleared": {
        text: "cleared",
        description: "Verb on a revision row: the editor (named before it) emptied a field. The field's name follows as a badge.",
    },
    "notes.revisions.changed": {
        text: "changed",
        description: "Verb on a revision row: the editor (named before it) edited a field. The field's name follows as a badge.",
    },
    "notes.revisions.unknownActor": {
        text: "Deleted account",
        description: "Shown instead of a name on a revision row whose author's account no longer exists.",
    },

    "notes.toasts.saved": {
        text: "Note saved",
        description: "Title of the confirmation toast after an operator note is saved.",
    },
    "notes.toasts.savedDesc": {
        text: "Updated operator notes for {name}.",
        description: "Body of the confirmation toast after saving. {name} is the operator's name.",
    },
    "notes.toasts.savedNext": {
        text: "{name}. Next up: {next}.",
        description: "Body of the confirmation toast after Save & next. {name} is the operator just saved, {next} the operator now open.",
    },
    "notes.toasts.failed": {
        text: "Couldn't save the note",
        description: "Title of the error toast when saving an operator note fails; the reason follows.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
