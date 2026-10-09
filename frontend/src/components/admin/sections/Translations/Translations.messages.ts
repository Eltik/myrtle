import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "admin";

export const messages = {
    "translations.head.kicker": {
        text: "Manage",
        description: "Small label above the Translations page title: the admin area this page belongs to.",
    },
    "translations.head.title": {
        text: "Translations",
        description: "Title of the admin page for translating the site's interface text.",
    },
    "translations.head.sub": {
        text: "Pick a language, then work through what's out of date or missing. {shortcut} saves and moves to the next string.",
        description: "Line under the Translations title. {shortcut} is the keyboard shortcut, e.g. '⌘↵' on a Mac or 'Ctrl↵' elsewhere.",
    },

    "translations.locale.hidden": {
        text: "Hidden",
        description: "Badge on a language card: the language exists but is not offered to visitors yet.",
    },
    "translations.locale.viewOnly": {
        text: "View only",
        description: "Badge on a language card: you can read this language's strings but not change them.",
    },
    "translations.locale.pct": {
        text: "{pct}%",
        description: "Share of a language's strings that are translated and up to date, on its card. {pct} is an already-formatted number such as '95.5'.",
    },
    "translations.locale.line": {
        text: "{stale} out of date · {missing} missing",
        description: "Line under a language's progress bar. {stale} and {missing} are formatted numbers. Keep the middle dot.",
    },
    "translations.locale.complete": {
        text: "Complete",
        description: "Line under a language's progress bar when nothing is out of date or missing.",
    },
    "translations.locale.none": {
        text: "There are no languages to translate yet.",
        description: "Shown in place of the language cards when the site has no languages besides English.",
    },

    "translations.list.search": {
        text: "Search…",
        description: "Placeholder in the search box above the list of strings. Keep it very short: the box can be under 100px wide. It matches the key, the English text and the translation.",
    },
    "translations.list.searchLabel": {
        text: "Search strings",
        description: "Accessible name of the search box above the list of strings.",
    },
    "translations.filter.label": {
        text: "Status",
        description: "Accessible name of the status dropdown (To do, Out of date, Missing, All) above the list of strings.",
    },
    "translations.filter.todo": {
        text: "To do",
        description: "Status dropdown option: strings that are out of date or missing. A count is shown next to it.",
    },
    "translations.filter.stale": {
        text: "Out of date",
        description: "Status dropdown option: strings whose English changed after they were translated. A count is shown next to it.",
    },
    "translations.filter.untranslated": {
        text: "Missing",
        description: "Status dropdown option: strings with no translation. A count is shown next to it.",
    },
    "translations.filter.all": {
        text: "All",
        description: "Status dropdown option that lists every string.",
    },
    "translations.area.label": {
        text: "Area",
        description: "Accessible name of the dropdown that narrows the list to one part of the site (a message namespace such as 'operators').",
    },
    "translations.area.all": {
        text: "All areas",
        description: "Area dropdown option (and its default display) that lists strings from every part of the site. Keep it short: it sits in a narrow dropdown.",
    },

    "translations.status.translated": {
        text: "Done",
        description: "Badge on a string that is translated and up to date.",
    },
    "translations.status.stale": {
        text: "Out of date",
        description: "Badge on a string whose English changed after it was translated.",
    },
    "translations.status.untranslated": {
        text: "Missing",
        description: "Badge on a string that has no translation yet.",
    },

    "translations.list.emptyFiltered": {
        text: "Nothing left here for {language}.",
        description: "Shown in the list when the chosen status has no strings. {language} is the language's English name.",
    },
    "translations.list.emptyAll": {
        text: "No strings match.",
        description: "Shown in the list when the search or area finds no strings at all.",
    },
    "translations.list.error": {
        text: "Couldn't load these strings.",
        description: "Shown in the list when it failed to load.",
    },
    "translations.list.retry": {
        text: "Try again",
        description: "Button that reloads the list after it failed to load.",
    },
    "translations.list.more": {
        text: "Load more",
        description: "Button at the bottom of the list that loads the next batch of strings.",
    },
    "translations.list.shown": {
        text: "{shown} of {total} shown",
        description: "Caption at the bottom of a long list. {shown} and {total} are formatted numbers.",
    },

    "translations.editor.back": {
        text: "All strings",
        description: "Button on phones that goes from the editor back to the list of strings.",
    },
    "translations.editor.title": {
        text: "{english} · {native}",
        description: "Editor heading: the language's English name and its own name, e.g. 'Japanese · 日本語'. Keep the middle dot.",
    },
    "translations.editor.english": {
        text: "English",
        description: "Label over the English text being translated.",
    },
    "translations.editor.writtenAgainst": {
        text: "Your translation was written against:",
        description: "Shown for an out-of-date string, before the earlier English marked up with what changed since.",
    },
    "translations.editor.diffNote": {
        text: "Struck-through words were removed from the English; underlined words were added.",
        description: "Caption explaining the markup of the comparison above it.",
    },
    "translations.editor.writtenAgainstUnknown": {
        text: "The English changed after this was translated, but the earlier version wasn't recorded. Re-read it and save to confirm.",
        description: "Shown for an out-of-date string saved before the site started keeping the earlier English.",
    },
    "translations.editor.context": {
        text: "Context:",
        description: "Bold label before the developer's note on where and how the string is used.",
    },
    "translations.editor.keep": {
        text: "Keep these exactly as written:",
        description: "Label before the chips listing the placeholders the translation must keep unchanged.",
    },
    "translations.editor.forms": {
        text: "Wordings",
        description: "Label over the tables that list the alternative wordings of a plural or choice placeholder.",
    },
    "translations.editor.translation": {
        text: "Translation",
        description: "Label over the box where the translation is written.",
    },
    "translations.editor.placeholder": {
        text: "Write the translation",
        description: "Placeholder in the empty translation box.",
    },
    "translations.editor.placeholderReadOnly": {
        text: "You have View access on this language, so this is read-only.",
        description: "Placeholder in the translation box when you may only read this language.",
    },
    "translations.editor.missing": {
        text: "{count, plural, one {Add {names}, including both braces, before saving. The site fills it in with a real value.} other {Add {names}, including the braces, before saving. The site fills them in with real values.}}",
        description: "Error under the translation box when it drops placeholders. {names} is a list such as '{title} and {when}'; {count} is how many.",
    },
    "translations.editor.position": {
        text: "{index} of {total}",
        description: "Footer of the editor: where this string sits in the list. Both are formatted numbers.",
    },
    "translations.editor.lastSaved": {
        text: "last saved {when}",
        description: "Footer of the editor, after a middle dot. {when} is a short relative time such as '2d ago'. Lowercase.",
    },
    "translations.editor.never": {
        text: "never translated",
        description: "Footer of the editor, after a middle dot, for a string with no translation. Lowercase.",
    },
    "translations.editor.revisions": {
        text: "{count, plural, one {# revision} other {# revisions}}",
        description: "Footer of the editor, after a middle dot: how many times this translation has been changed.",
    },
    "translations.editor.skip": {
        text: "Skip",
        description: "Button that moves to the next string without saving.",
    },
    "translations.editor.saveNext": {
        text: "Save & next",
        description: "Button that saves the translation and opens the next string. A keyboard shortcut is shown beside it.",
    },
    "translations.editor.markCurrent": {
        text: "Mark as current",
        description: "Replaces the save button when the source text changed but the translation was left as is: confirms the translation still fits the new source and opens the next string. A keyboard shortcut is shown beside it.",
    },
    "translations.editor.more": {
        text: "More actions",
        description: "Accessible name of the '⋯' button that opens the editor's extra actions.",
    },
    "translations.editor.history": {
        text: "History",
        description: "Menu item that opens every earlier version of this translation.",
    },
    "translations.editor.discard": {
        text: "Discard edits",
        description: "Menu item that throws away unsaved typing and puts back the saved translation.",
    },
    "translations.editor.clear": {
        text: "Clear translation",
        description: "Menu item that deletes this translation, so the site shows English again.",
    },

    "translations.args.kind.plain": {
        text: "text",
        description: "Label on a placeholder whose value is dropped in as-is, such as a name. Lowercase, shown on a small chip.",
    },
    "translations.args.kind.number": {
        text: "number",
        description: "Label on a placeholder holding a number. Lowercase, shown on a small chip.",
    },
    "translations.args.kind.date": {
        text: "date",
        description: "Label on a placeholder holding a date. Lowercase, shown on a small chip.",
    },
    "translations.args.kind.time": {
        text: "time",
        description: "Label on a placeholder holding a time of day. Lowercase, shown on a small chip.",
    },
    "translations.args.kind.plural": {
        text: "plural",
        description: "Label on a placeholder that picks a wording by how many there are. Lowercase, shown on a small chip.",
    },
    "translations.args.kind.selectordinal": {
        text: "ordinal",
        description: "Label on a placeholder that picks a wording by rank: 1st, 2nd, 3rd. Lowercase, shown on a small chip.",
    },
    "translations.args.kind.select": {
        text: "choice",
        description: "Label on a placeholder that picks a wording from a fixed set of named cases. Lowercase, shown on a small chip.",
    },
    "translations.args.pluralIntro": {
        text: "The site picks one of these wordings by the number. {language} uses {count}.",
        description: "Explains the table under a plural placeholder. {language} is the language name; {count} is an already-worded count such as '4 forms'.",
    },
    "translations.args.pluralCount": {
        text: "{count, plural, one {# form} other {# forms}}",
        description: "The {count} written into the sentence above it. Use your language's own plural forms here.",
    },
    "translations.args.selectIntro": {
        text: "The site picks one of these wordings by which case applies. Keep every case the English has.",
        description: "Explains the table under a choice placeholder, where the cases are named rather than counted.",
    },
    "translations.args.colForm": {
        text: "Form",
        description: "Column heading over the names of a placeholder's wordings: one, few, other.",
    },
    "translations.args.colApplies": {
        text: "Applies to",
        description: "Column heading over the numbers that select each wording.",
    },
    "translations.args.colEnglish": {
        text: "English",
        description: "Column heading over the English wording for each form.",
    },
    "translations.args.exact": {
        text: "exactly {value}",
        description: "What an '=0' style form applies to. {value} is the number it matches exactly.",
    },
    "translations.args.anythingElse": {
        text: "anything else",
        description: "What the 'other' form applies to: every number no earlier form claimed.",
    },
    "translations.args.formMissing": {
        text: "Missing",
        description: "Badge on a form the language needs but the translation does not have yet, which makes the site fall back to 'other'.",
    },

    "translations.done.title": {
        text: "All caught up in {language}",
        description: "Shown in place of the editor when the chosen status has nothing left. {language} is the language's English name.",
    },
    "translations.done.desc": {
        text: "Switch language above, or look at every string.",
        description: "Line under 'All caught up'.",
    },
    "translations.done.showAll": {
        text: "Show all strings",
        description: "Button that lists every string in this language, whatever its status or area.",
    },
    "translations.noMatch.title": {
        text: "No strings match",
        description: "Shown in place of the editor when the search or area finds nothing.",
    },
    "translations.noMatch.desc": {
        text: "Try another search, or look in every area.",
        description: "Line under 'No strings match'.",
    },
    "translations.noMatch.reset": {
        text: "Clear search",
        description: "Button that clears the search box and the area filter.",
    },

    "translations.history.title": {
        text: "History",
        description: "Title of the side panel listing earlier versions of one translation.",
    },
    "translations.history.desc": {
        text: "{key} · {language}",
        description: "Line under the History title. {key} is the string's id, {language} the language's English name. Keep the middle dot.",
    },
    "translations.history.empty": {
        text: "Nobody has changed this translation yet.",
        description: "Shown in the History panel when there are no earlier versions.",
    },
    "translations.history.error": {
        text: "Couldn't load the history.",
        description: "Shown in the History panel when it failed to load.",
    },
    "translations.history.by": {
        text: "by {actor}",
        description: "Who made a change, after its time. {actor} is the account's nickname, or the deleted-account label.",
    },
    "translations.history.deletedAccount": {
        text: "Deleted account",
        description: 'Stands in for the author of a History row whose account no longer exists. Fills {actor} in "by {actor}".',
    },
    "translations.history.wasEmpty": {
        text: "(was untranslated)",
        description: "Shown in place of the earlier value when there was no translation before. Keep the parentheses.",
    },
    "translations.history.cleared": {
        text: "(cleared)",
        description: "Shown in place of the new value when the change removed the translation. Keep the parentheses.",
    },
    "translations.history.revert": {
        text: "Revert to this",
        description: "Button on a History row that writes the value it replaced back.",
    },

    "translations.toast.saved": {
        text: "Translation saved",
        description: "Toast title after a translation was saved.",
    },
    "translations.toast.saved.desc": {
        text: "{key} · {language}",
        description: "Toast body after a save. {key} is the string's id, {language} the language's English name. Keep the middle dot.",
    },
    "translations.toast.saveFailed": {
        text: "Couldn't save the translation",
        description: "Toast title when saving a translation failed.",
    },
    "translations.toast.cleared": {
        text: "Translation cleared",
        description: "Toast title after a translation was deleted.",
    },
    "translations.toast.cleared.desc": {
        text: "{key} shows English in {language} again.",
        description: "Toast body after a translation was deleted. {key} is the string's id, {language} the language's English name.",
    },
    "translations.toast.clearFailed": {
        text: "Couldn't clear the translation",
        description: "Toast title when deleting a translation failed.",
    },
    "translations.toast.reverted": {
        text: "Translation reverted",
        description: "Toast title after an earlier version was written back.",
    },
    "translations.toast.reverted.desc": {
        text: "{key} is back to an earlier version.",
        description: "Toast body after a revert. {key} is the string's id.",
    },
    "translations.toast.revertFailed": {
        text: "Couldn't revert the translation",
        description: "Toast title when writing an earlier version back failed.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
