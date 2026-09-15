import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "admin";

export const messages = {
    "i18n.kicker": {
        text: "Manage",
        description: "Eyebrow over the Translations page title, naming the admin section it belongs to. Rendered uppercase.",
    },
    "i18n.title": {
        text: "Translations",
        description: "Title of the admin screen where the interface's own text is translated.",
    },
    "i18n.sub": {
        text: "The UI message catalog. Saves write {table} and append the replaced value to {auditTable}, which is what makes revert an ordinary write.",
        description: "Sentence under the Translations title: because the old value is kept, undoing an edit is just another save. {table} and {auditTable} are database table names, shown in monospace. 'UI' is the user interface.",
    },
    "i18n.refresh": {
        text: "Refresh",
        description: "Button that re-fetches the message list and the progress figures.",
    },
    "i18n.noLocales": {
        text: "No locales",
        description: "Disabled menu item shown when the locale picker has nothing to offer.",
    },
    "i18n.readOnlyTag": {
        text: "read-only",
        description: "Marker beside a locale in the picker that this admin may read but not write.",
    },
    "i18n.nsLabel": {
        text: "ns",
        description: "Terse label on the namespace picker. 'ns' abbreviates namespace, the feature area a message key belongs to.",
    },
    "i18n.nsAll": {
        text: "all",
        description: "Value shown on the namespace picker when no namespace filter is applied.",
    },
    "i18n.allNamespaces": {
        text: "All namespaces",
        description: "Menu item that clears the namespace filter.",
    },
    "i18n.searchPlaceholder": {
        text: "Search key, English source, or translation…",
        description: "Prompt in the message search box. A 'key' is the message's identifier. Keep the ellipsis character.",
    },
    "i18n.keyCount": {
        text: "{count} keys",
        description: "How many message keys the current filter matches. The number is already formatted; the wording stays plural at any count.",
    },
    "i18n.filter.all": {
        text: "All",
        description: "Filter chip showing every message key.",
    },
    "i18n.filter.untranslated": {
        text: "Untranslated",
        description: "Filter chip showing only keys with no translation in this locale.",
    },
    "i18n.filter.stale": {
        text: "Stale",
        description: "Filter chip showing only keys whose English source changed after they were translated.",
    },
    "i18n.filter.translated": {
        text: "Translated",
        description: "Filter chip showing only keys with an up-to-date translation.",
    },
    "i18n.noGrant": {
        text: "You hold no edit grant on {locale} - read-only.",
        description: "Read-only notice above the message list. {locale} is the locale code, shown in monospace. Keep the hyphen.",
    },
    "i18n.loadError": {
        text: "Couldn't load the message list.",
        description: "Shown in place of the message list when the request failed.",
    },
    "i18n.noMatch": {
        text: "No keys match these filters.",
        description: "Empty state when keys exist but none survive the current filters. A 'Reset' button follows it.",
    },
    "i18n.reset": {
        text: "Reset",
        description: "Button that clears the message search, namespace and state filters.",
    },
    "i18n.empty": {
        text: "No message keys in this locale yet - run the extractor sync.",
        description: "Empty state when the locale has no keys at all. The 'extractor sync' is the build step that pushes keys to the backend. Keep the hyphen.",
    },
    "i18n.progress.empty": {
        text: "No locales configured yet.",
        description: "Empty state in place of the per-locale progress tiles.",
    },
    "i18n.progress.ratio": {
        text: "{translated} / {total}",
        description: "How many of a locale's keys are translated, under the percentage. Both numbers are already formatted. Keep the slash.",
    },
    "i18n.progress.stale": {
        text: "{count} stale",
        description: "Badge counting a locale's keys whose English source changed after translation. The number is already formatted.",
    },
    "i18n.progress.untranslated": {
        text: "{count} untranslated",
        description: "Badge counting a locale's keys with no translation. The number is already formatted.",
    },
    "i18n.progress.complete": {
        text: "complete",
        description: "Badge on a locale with nothing left to translate and nothing stale.",
    },
    "i18n.notTranslated": {
        text: "Not translated yet.",
        description: "Placeholder in a message row where the translation would be.",
    },
    "i18n.badge.stale": {
        text: "stale",
        description: "Badge on a message whose English source changed after it was translated.",
    },
    "i18n.badge.untranslated": {
        text: "untranslated",
        description: "Badge on a message with no translation in this locale.",
    },
    "i18n.badge.translated": {
        text: "translated",
        description: "Badge on a message with an up-to-date translation.",
    },
    "i18n.editor.staleNote": {
        text: "The English source changed after this translation was written - re-read the source and save to re-stamp it.",
        description: "Caption over the editor for a stale message. 'Re-stamp' means saving records the new source as the one translated. Keep the hyphen.",
    },
    "i18n.editor.lastSaved": {
        text: "Last saved {when}",
        description: "Caption over the editor; {when} is an already-formatted relative time such as '3 days ago'.",
    },
    "i18n.editor.never": {
        text: "Never translated in this locale.",
        description: "Caption over the editor for a message that has never been written in this locale.",
    },
    "i18n.history": {
        text: "History",
        description: "Button that opens the list of past values for this message.",
    },
    "i18n.close": {
        text: "Close",
        description: "Button that closes the message editor.",
    },
    "i18n.save": {
        text: "Save",
        description: "Button that writes the translation.",
    },
    "i18n.editor.translationLabel": {
        text: "Translation ·",
        description: "Label over the translation box; the locale code follows after a space. Keep the middle dot.",
    },
    "i18n.unsaved": {
        text: "unsaved changes",
        description: "Badge over the translation box when it holds edits that have not been written yet.",
    },
    "i18n.saved": {
        text: "saved",
        description: "Badge over the translation box when it matches what is stored.",
    },
    "i18n.editor.placeholder": {
        text: "Write the translation…",
        description: "Prompt inside the empty translation box. Keep the ellipsis character.",
    },
    "i18n.editor.placeholderReadOnly": {
        text: "You have no edit grant on this locale.",
        description: "Shown inside the disabled translation box for an admin who may only read this locale.",
    },
    "i18n.editor.placeholderRule": {
        text: "Every declared placeholder must appear, and no others - the save is rejected otherwise.",
        description: "Helper text under the translation box. A 'placeholder' is a {token} the message declares. Keep the hyphen.",
    },
    "i18n.clear": {
        text: "Clear translation",
        description: "Destructive button that removes the translation, putting the key back to untranslated.",
    },
    "i18n.discard": {
        text: "Discard edits",
        description: "Button that throws away unsaved changes in the translation box.",
    },
    "i18n.englishSource": {
        text: "English source",
        description: "Heading over the original text the translation is made from. Rendered uppercase.",
    },
    "i18n.context": {
        text: "Context",
        description: "Heading over the note the message's author wrote for translators. Rendered uppercase.",
    },
    "i18n.placeholders": {
        text: "Placeholders",
        description: "Heading over the {token} arguments the message declares. Rendered uppercase.",
    },
    "i18n.placeholders.none": {
        text: "None - this message takes no arguments.",
        description: "Shown under the Placeholders heading when the message has none. Keep the hyphen.",
    },
    "i18n.sourceHash": {
        text: "Source hash",
        description: "Heading over the fingerprint of the English text this translation was made from. Rendered uppercase.",
    },
    "i18n.toast.saved": {
        text: "Translation saved",
        description: "Toast title after a translation was written.",
    },
    "i18n.toast.saved.desc": {
        text: "{key} · {locale}",
        description: "Toast body after a translation was written: the message key and the locale code, both identifiers. Keep the middle dot.",
    },
    "i18n.toast.saveFailed": {
        text: "Failed to save",
        description: "Toast title after a translation write was rejected.",
    },
    "i18n.toast.cleared": {
        text: "Translation cleared",
        description: "Toast title after a translation was removed.",
    },
    "i18n.toast.cleared.desc": {
        text: "{key} is back to untranslated.",
        description: "Toast body after a translation was removed; {key} is the message key, an identifier.",
    },
    "i18n.toast.clearFailed": {
        text: "Failed to clear",
        description: "Toast title after removing a translation was rejected.",
    },
    "i18n.closeDrawer": {
        text: "Close drawer",
        description: "Accessible name of the backdrop behind the history drawer; clicking it closes the drawer.",
    },
    "i18n.history.kicker": {
        text: "History · {locale}",
        description: "Eyebrow at the top of the history drawer; {locale} is the locale code. Keep the middle dot. Rendered uppercase.",
    },
    "i18n.history.count": {
        text: "{count, plural, one {# revision} other {# revisions}} · every row stores the value it replaced",
        description: "Caption in the history drawer: how many past values there are and what each row keeps. Keep the middle dot.",
    },
    "i18n.history.empty": {
        text: "No revisions yet.",
        description: "Empty state in the history drawer.",
    },
    "i18n.history.changedKey": {
        text: "{actor} changed this key",
        description: "One revision row in the history drawer. {actor} is the editor's truncated account id, shown in monospace.",
    },
    "i18n.history.wasUntranslated": {
        text: "(was untranslated)",
        description: "Shown in place of the replaced value when the key had no translation before. Keep the parentheses.",
    },
    "i18n.history.cleared": {
        text: "(cleared)",
        description: "Shown in place of the new value when the edit removed the translation. Keep the parentheses.",
    },
    "i18n.history.revert": {
        text: "Revert to this",
        description: "Button on a history row that writes that older value back.",
    },
    "i18n.toast.revertRejected": {
        text: "Revert rejected",
        description: "Toast title when writing an older value back failed validation.",
    },
    "i18n.toast.reverted": {
        text: "Reverted",
        description: "Toast title after an older value was written back.",
    },
    "i18n.toast.reverted.desc": {
        text: "{key} restored to an earlier value.",
        description: "Toast body after an older value was written back; {key} is the message key, an identifier.",
    },
    "i18n.toast.revertFailed": {
        text: "Failed to revert",
        description: "Toast title after writing an older value back was rejected.",
    },
    "i18n.grants.title": {
        text: "Locale grants ·",
        description: "Heading of the card listing who may write this locale; the locale code follows after a space. Keep the middle dot.",
    },
    "i18n.grants.desc": {
        text: "Rows in {table}. Read per request, so a new grant works on the very next call - unlike the global role, which rides the JWT.",
        description: "Caption under the Locale grants heading: grants take effect immediately, whereas the site-wide role waits for a new sign-in token. {table} is a database table name, shown in monospace. 'JWT' is the token format and stays as-is.",
    },
    "i18n.grants.add": {
        text: "Grant locale",
        description: "Button that opens the dialog for giving someone write access to this locale.",
    },
    "i18n.grants.empty": {
        text: "No grants on this locale yet - only super-admins can write it.",
        description: "Empty state in the locale grants table. Keep the hyphen.",
    },
    "i18n.th.user": {
        text: "User",
        description: "Grants table column header for who holds the grant. Rendered uppercase.",
    },
    "i18n.th.level": {
        text: "Level",
        description: "Grants table column header for which rung of the ladder the grant is. Rendered uppercase.",
    },
    "i18n.th.granted": {
        text: "Granted",
        description: "Grants table column header for when the grant was made. Rendered uppercase.",
    },
    "i18n.th.grantedBy": {
        text: "Granted by",
        description: "Grants table column header for the admin who made the grant. Rendered uppercase.",
    },
    "i18n.revoke": {
        text: "Revoke",
        description: "Button that removes one locale grant. A verb.",
    },
    "i18n.toast.grantRevoked": {
        text: "Grant revoked",
        description: "Toast title after a locale grant was removed.",
    },
    "i18n.toast.grantRevoked.desc": {
        text: "Removed a grant on {locale}.",
        description: "Toast body after a locale grant was removed; {locale} is the locale code.",
    },
    "i18n.toast.revokeFailed": {
        text: "Failed to revoke",
        description: "Toast title after removing a grant was rejected.",
    },
    "i18n.grant.kicker": {
        text: "Manage · translations",
        description: "Eyebrow at the top of the locale grant dialog. Keep the middle dot. Rendered uppercase.",
    },
    "i18n.grant.title": {
        text: "Grant access on {locale}",
        description: "Heading of the locale grant dialog; {locale} is the locale code.",
    },
    "i18n.grant.findDoctor": {
        text: "Find a Doctor",
        description: "Label over the account search in the grant dialog. 'Doctor' is what Arknights calls the player.",
    },
    "i18n.grant.searchPlaceholder": {
        text: "Search by nickname…",
        description: "Prompt in the account search box of the grant dialog. Keep the ellipsis character.",
    },
    "i18n.grant.noMatches": {
        text: "No matches.",
        description: "Shown under the account search in the grant dialog when nothing matched.",
    },
    "i18n.uid": {
        text: "UID {uid}",
        description: "An account's in-game number beside its nickname. 'UID' is the game's own abbreviation and stays as-is.",
    },
    "i18n.grant.level": {
        text: "Level",
        description: "Label over the rung picker in the grant dialog.",
    },
    "i18n.grant.levelHint": {
        text: "The target account still needs the global translator role; the grant only says which locales.",
        description: "Helper text under the rung picker: the grant alone is not enough without the site-wide translator role.",
    },
    "i18n.cancel": {
        text: "Cancel",
        description: "Button that closes the grant dialog without saving.",
    },
    "i18n.grant.submit": {
        text: "Grant {level}",
        description: "Confirm button of the grant dialog; {level} is the chosen rung, e.g. 'Grant Edit'.",
    },
    "i18n.toast.grantCreated": {
        text: "Grant created",
        description: "Toast title after a new locale grant was made.",
    },
    "i18n.toast.grantCreated.desc": {
        text: "{level} on {locale}.",
        description: "Toast body after a new locale grant was made; {level} is the rung and {locale} the locale code.",
    },
    "i18n.toast.grantFailed": {
        text: "Failed to grant",
        description: "Toast title after a new locale grant was rejected.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
