import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "admin";

export const messages = {
    "locales.server.en": {
        text: "Global (EN)",
        description: "Game-region option: the worldwide English client. 'EN' is its own code and stays as-is.",
    },
    "locales.server.jp": {
        text: "Japan (JP)",
        description: "Game-region option: the Japanese client. 'JP' is its own code and stays as-is.",
    },
    "locales.server.kr": {
        text: "Korea (KR)",
        description: "Game-region option: the Korean client. 'KR' is its own code and stays as-is.",
    },
    "locales.server.cn": {
        text: "Mainland China (CN)",
        description: "Game-region option: the mainland Chinese client. 'CN' is its own code and stays as-is.",
    },
    "locales.server.tw": {
        text: "Taiwan (TW)",
        description: "Game-region option: the Taiwanese client. 'TW' is its own code and stays as-is.",
    },
    "locales.server.bili": {
        text: "Bilibili (CN)",
        description: "Game-region option: the Bilibili-published Chinese client. 'Bilibili' is the publisher's name and 'CN' its region code; both stay as-is.",
    },
    "locales.err.code.required": {
        text: "Required - this is the primary key and the URL prefix.",
        description: "Validation message when the locale code is blank. Keep the hyphen.",
    },
    "locales.err.code.shape": {
        text: "Must match {pattern} - anything else never routes.",
        description: "Validation message when the locale code is not a valid tag; {pattern} is the regular expression it must match. 'Routes' means the site's router will never recognise the URL. Keep the hyphen.",
    },
    "locales.err.code.exists": {
        text: "{code} already exists - edit that row instead.",
        description: "Validation message when adding a locale code that is already configured. Keep the hyphen.",
    },
    "locales.err.englishName": {
        text: "Required - this is how the locale is listed in the admin panel.",
        description: "Validation message when the English name is blank. Keep the hyphen.",
    },
    "locales.err.nativeName": {
        text: "Required - this is the label in the public language switcher.",
        description: "Validation message when the native name is blank. Keep the hyphen.",
    },
    "locales.err.fallback.source": {
        text: "{locale} is the source locale - it is the bottom of every fallback chain and cannot have one.",
        description: "Validation message when a fallback is picked for the source locale; {locale} is its code. Keep the hyphen.",
    },
    "locales.err.fallback.self": {
        text: "A locale cannot fall back to itself.",
        description: "Validation message when the chosen fallback is the locale being edited.",
    },
    "locales.err.fallback.cycle": {
        text: "That choice closes a fallback cycle, which the resolver cannot terminate.",
        description: "Validation message when the chosen fallback would make the chain loop forever.",
    },
    "locales.err.gamedataServer": {
        text: "Must be one of {servers}.",
        description: "Validation message when the game region is not one the backend knows; {servers} is the comma-separated list of valid region codes.",
    },
    "locales.err.enabled": {
        text: "{locale} is the source locale and the fallback beneath every other - it cannot be disabled.",
        description: "Validation message when the source locale is switched off; {locale} is its code. Keep the hyphen.",
    },
    "locales.err.sortOrder": {
        text: "A whole number, 0 or greater. Lower sorts first in the switcher.",
        description: "Validation message when the sort order is not a non-negative integer.",
    },
    "locales.title": {
        text: "Locales",
        description: "Heading of the card that lists and edits the site's configured languages.",
    },
    "locales.desc": {
        text: "Rows in {table}. The code is both the primary key and the URL prefix; {column} decides which Arknights client supplies operator, skill and stage text under that prefix.",
        description: "Caption under the Locales heading. {table} is a database table name and {column} one of its columns, both shown in monospace. 'Operator', 'skill' and 'stage' are game concepts whose text comes from the game itself.",
    },
    "locales.desc.readOnly": {
        text: " You are not a super-admin, so this list is read-only.",
        description: "Appended to the caption under the Locales heading for an admin who cannot edit locales. Keep the leading space.",
    },
    "locales.add": {
        text: "Add locale",
        description: "Button that opens the dialog for configuring a new language.",
    },
    "locales.empty": {
        text: "No locales configured yet.",
        description: "Empty state in the Locales card.",
    },
    "locales.th.code": {
        text: "Code",
        description: "Locale table column header for the language tag. Rendered uppercase.",
    },
    "locales.th.name": {
        text: "Name",
        description: "Locale table column header for the language's own name and its English name. Rendered uppercase.",
    },
    "locales.th.fallback": {
        text: "Fallback",
        description: "Locale table column header for which locale an untranslated key falls back to. Rendered uppercase.",
    },
    "locales.th.gameData": {
        text: "Game data",
        description: "Locale table column header for which Arknights client supplies game text. Rendered uppercase.",
    },
    "locales.th.sort": {
        text: "Sort",
        description: "Locale table column header for the ordering number. Rendered uppercase.",
    },
    "locales.th.translated": {
        text: "Translated",
        description: "Locale table column header for how much of the catalog is translated. Rendered uppercase.",
    },
    "locales.th.state": {
        text: "State",
        description: "Locale table column header for whether the locale is public. Rendered uppercase.",
    },
    "locales.source": {
        text: "source",
        description: "Marker beside the locale the English text is written in.",
    },
    "locales.badge.enabled": {
        text: "enabled",
        description: "Badge on a locale that is public.",
    },
    "locales.badge.hidden": {
        text: "hidden",
        description: "Badge on a locale that is configured but not public yet.",
    },
    "locales.edit": {
        text: "Edit",
        description: "Button on a locale row that opens the locale dialog. A verb.",
    },
    "locales.close": {
        text: "Close",
        description: "Accessible name of the backdrop behind the locale dialog; clicking it closes the dialog.",
    },
    "locales.dialog.kicker": {
        text: "Manage · locales",
        description: "Eyebrow at the top of the locale dialog. Keep the middle dot. Rendered uppercase.",
    },
    "locales.dialog.add": {
        text: "Add a locale",
        description: "Heading of the locale dialog when creating a new locale.",
    },
    "locales.dialog.edit": {
        text: "Edit {code}",
        description: "Heading of the locale dialog when changing an existing locale; {code} is its language tag.",
    },
    "locales.form.code": {
        text: "Code",
        description: "Label over the locale's language-tag field.",
    },
    "locales.form.code.hint": {
        text: "A BCP-47-ish tag and the URL prefix. The router treats a leading path segment matching {pattern} as a locale claim, so a code outside that shape will never route - it falls through to a real route or a 404.",
        description: "Helper text under the code field. {pattern} is the required pattern, shown in monospace. 'BCP-47' is the standard for language tags; '404' is the not-found response. Keep the hyphen.",
    },
    "locales.form.code.locked": {
        text: "The code is the primary key and cannot be changed - add a new locale instead.",
        description: "Note under the code field when editing an existing locale. Keep the hyphen.",
    },
    "locales.form.englishName": {
        text: "English name",
        description: "Label over the field holding the language's name in English.",
    },
    "locales.form.englishName.placeholder": {
        text: "Japanese",
        description: "Example inside the empty English-name field: the name of a language, written in English.",
    },
    "locales.form.englishName.hint": {
        text: "How this locale is listed in the admin panel.",
        description: "Helper text under the English-name field.",
    },
    "locales.form.nativeName": {
        text: "Native name",
        description: "Label over the field holding the language's name in its own language.",
    },
    "locales.form.nativeName.placeholder": {
        text: "日本語",
        description: "Example inside the empty native-name field: the word 'Japanese' written in Japanese. Substitute an example in the language you are translating into.",
    },
    "locales.form.nativeName.hint": {
        text: "The label the public language switcher shows, in its own language.",
        description: "Helper text under the native-name field.",
    },
    "locales.form.fallback": {
        text: "Fallback locale",
        description: "Label over the picker for where an untranslated key looks next.",
    },
    "locales.form.fallback.hintSource": {
        text: "{locale} is the source locale: it is the bottom of every fallback chain, so it takes no fallback of its own.",
        description: "Helper text under the fallback picker when editing the source locale. {locale} is that locale's code, shown in monospace.",
    },
    "locales.form.fallback.hint": {
        text: "Where an untranslated key looks next, before the bundled English source.",
        description: "Helper text under the fallback picker.",
    },
    "locales.form.fallback.none": {
        text: "None",
        description: "Fallback option meaning this locale falls straight through to the bundled English source.",
    },
    "locales.form.enabled.hintSource": {
        text: "{locale} is the source locale, so it is always public - the server keeps it enabled whatever this form sends.",
        description: "Helper text beside the disabled enabled-toggle when editing the source locale. {locale} is that locale's code, shown in monospace. Keep the hyphen.",
    },
    "locales.form.gamedataServer": {
        text: "Game data server",
        description: "Label over the picker for which Arknights client supplies game text.",
    },
    "locales.form.gamedataServer.hint": {
        text: "Which Arknights client supplies operator names, skill descriptions and stage text for this locale. Translating the chrome does not translate game data - this is what does.",
        description: "Helper text under the game-region picker. 'Chrome' here means the site's own interface text, as opposed to text from the game. Keep the hyphen.",
    },
    "locales.form.gamedataServer.placeholder": {
        text: "Pick a region",
        description: "Prompt in the game-region picker before anything is chosen.",
    },
    "locales.form.sortOrder": {
        text: "Sort order",
        description: "Label over the field that orders locales in the public language switcher.",
    },
    "locales.form.sortOrder.hint": {
        text: "Lower sorts first in the public language switcher.",
        description: "Helper text under the sort-order field.",
    },
    "locales.form.enabled": {
        text: "Enabled",
        description: "Label over the switch that makes a locale public. Rendered uppercase.",
    },
    "locales.thisLocale": {
        text: "this locale",
        description: "Stand-in for the locale's code before one has been typed, e.g. 'Enabling makes this locale public'.",
    },
    "locales.enable.desc": {
        text: "Enabling makes {code} public: it appears in the language switcher, {path} starts routing, and the deployment is committed to the {server} game region's data plus a font subset for its script.",
        description: "Note beside the Enabled switch. {code} is the locale's code, {path} its URL prefix and {server} the game-region code, all shown in monospace. A 'font subset' is the slice of a typeface covering that writing system.",
    },
    "locales.warn": {
        text: "The backend must also list {server} in its {env} env, or every {path} game-data request 404s and the locale renders translated chrome over empty data.",
        description: "Warning under the Enabled switch: requests fail with a not-found response. {server} is the game-region code, {env} an environment-variable name and {path} the game-data path, all shown in monospace. 'env' abbreviates environment; 'chrome' here means the site's own interface text.",
    },
    "locales.confirm.title": {
        text: "Make {code} public?",
        description: "Heading of the confirmation step before a locale is switched on; {code} is its language tag, or a stand-in before one is typed.",
    },
    "locales.confirm.body": {
        text: "Visitors will be able to switch to it and {path} URLs become shareable. Untranslated keys fall back {fallback}, so a locale enabled early ships visibly mixed text.",
        description: "Body of the enable confirmation. {path} is the locale's URL prefix, shown in monospace; {fallback} says where untranslated keys fall back to and is one of the two 'fall back' phrases in this namespace.",
    },
    "locales.confirm.fallbackEnglish": {
        text: "straight to English",
        description: "Fills {fallback} in the enable confirmation when the locale has no fallback: 'Untranslated keys fall back straight to English'.",
    },
    "locales.confirm.fallbackChain": {
        text: "to {locale}, then English",
        description: "Fills {fallback} in the enable confirmation when the locale has a fallback; {locale} is that fallback's code: 'Untranslated keys fall back to pt, then English'.",
    },
    "locales.confirm.yes": {
        text: "Yes, enable it",
        description: "Button that confirms making the locale public.",
    },
    "locales.confirm.no": {
        text: "Keep it hidden",
        description: "Button that cancels making the locale public.",
    },
    "locales.fixFields": {
        text: "Fix the highlighted fields first.",
        description: "Note beside the save button when the locale form has validation errors.",
    },
    "locales.cancel": {
        text: "Cancel",
        description: "Button that closes the locale dialog without saving.",
    },
    "locales.create": {
        text: "Create locale",
        description: "Confirm button of the locale dialog when adding a new locale.",
    },
    "locales.save": {
        text: "Save",
        description: "Confirm button of the locale dialog when changing an existing locale.",
    },
    "locales.toast.created": {
        text: "Locale created",
        description: "Toast title after a new locale was configured.",
    },
    "locales.toast.saved": {
        text: "Locale saved",
        description: "Toast title after an existing locale was changed.",
    },
    "locales.toast.saved.desc": {
        text: "{code} · {name}",
        description: "Toast body after a locale was written: its language tag and its native name. Keep the middle dot.",
    },
    "locales.toast.failed": {
        text: "Failed to save locale",
        description: "Toast title after a locale write was rejected.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
