import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "admin";

export const messages = {
    "system.head.kicker": {
        text: "Operate",
        description: "Small label above the System page title: the admin area for running the site, as opposed to managing content.",
    },
    "system.head.title": {
        text: "System",
        description: "Title of the admin page for service health, the edit history and site languages.",
    },
    "system.head.sub": {
        text: "Service health, the edit history and site languages, in one place.",
        description: "Line under the System page title listing its three tabs.",
    },
    "system.head.checkNow": {
        text: "Check now",
        description: "Button in the System page header that probes the database and cache again right away.",
    },

    "system.tabs.health": {
        text: "Health",
        description: "System page tab showing whether the database and cache respond, and what data the backend holds.",
    },
    "system.tabs.audit": {
        text: "Edit history",
        description: "System page tab listing recent edits to operator notes and translations.",
    },
    "system.tabs.languages": {
        text: "Languages",
        description: "System page tab listing the site's languages and whether visitors can pick them.",
    },

    "system.toast.healthy.title": {
        text: "Health check passed",
        description: "Toast title after a manual health check where every service responded.",
    },
    "system.toast.healthy.desc": {
        text: "Database and cache responded.",
        description: "Toast body after a manual health check where every service responded.",
    },
    "system.toast.unhealthy.title": {
        text: "Health check found a problem",
        description: "Toast title after a manual health check where a service is down. The body names which one.",
    },
    "system.toast.failed.title": {
        text: "Health check failed",
        description: "Toast title when the health check request itself failed, so nothing is known about the services.",
    },

    "system.health.status.checking": {
        text: "Checking services…",
        description: "Status line on the Health tab while the first probe is still running.",
    },
    "system.health.status.healthy": {
        text: "All services healthy",
        description: "Status line on the Health tab when the database and cache both respond.",
    },
    "system.health.status.cacheDown": {
        text: "Cache isn't responding",
        description: "Status line on the Health tab when the cache service is down but the database works.",
    },
    "system.health.status.databaseDown": {
        text: "Database isn't responding",
        description: "Status line on the Health tab when the database is down.",
    },
    "system.health.status.bothDown": {
        text: "Database and cache aren't responding",
        description: "Status line on the Health tab when both the database and the cache are down.",
    },
    "system.health.status.degraded": {
        text: "The backend reports a problem",
        description: "Status line on the Health tab when both services respond but the backend still reports itself degraded.",
    },
    "system.health.status.unreachable": {
        text: "Couldn't reach the backend",
        description: "Status line on the Health tab when the health check request itself failed.",
    },
    "system.health.detail.cacheDown": {
        text: "Reads fall through to the database, so the site is slower but still works.",
        description: "Explanation under the status line (and in the toast) when only the cache is down.",
    },
    "system.health.detail.databaseDown": {
        text: "Anything that reads or saves accounts, notes or tier lists will fail until it's back.",
        description: "Explanation under the status line (and in the toast) when the database is down.",
    },
    "system.health.detail.bothDown": {
        text: "Most pages will fail until the database is back.",
        description: "Explanation under the status line (and in the toast) when both the database and the cache are down.",
    },
    "system.health.detail.degraded": {
        text: "Both services answered, but the backend still marks itself degraded. Check its logs.",
        description: "Explanation under the status line (and in the toast) when the backend reports a problem the two probes don't show.",
    },
    "system.health.detail.unreachable": {
        text: "The health check itself failed, so the numbers below may be out of date.",
        description: "Explanation under the status line when the health check request failed.",
    },
    "system.health.checked": {
        text: "Checked {when} · refreshes every 30s",
        description: "Muted note beside the status line. {when} is a short relative time such as '12s ago'. 's' abbreviates seconds. Keep the middle dot.",
    },
    "system.health.tile.database": {
        text: "Database",
        description: "Label of the tile showing how long the database took to answer the health probe.",
    },
    "system.health.tile.cache": {
        text: "Cache · {backend}",
        description: "Label of the tile showing how long the cache took to answer the health probe. {backend} is the cache kind, such as 'Redis'. Keep the middle dot.",
    },
    "system.health.tile.roundTrip": {
        text: "Round-trip",
        description: "Label of the tile showing how long the whole health request took.",
    },
    "system.health.unit.ms": {
        text: "ms",
        description: "Unit after a response time: milliseconds.",
    },
    "system.health.cacheBackend.redis": {
        text: "Redis",
        description: "Cache kind shown in the cache tile label. 'Redis' is a product name and stays as-is.",
    },
    "system.health.cacheBackend.memory": {
        text: "in memory",
        description: "Cache kind shown in the cache tile label when the backend caches in its own memory instead of a separate service.",
    },
    "system.health.site.title": {
        text: "Site data",
        description: "Heading of the card counting rosters and tier lists stored on the site.",
    },
    "system.health.site.rosters": {
        text: "Rosters synced",
        description: "Count label: player rosters imported from the game.",
    },
    "system.health.site.tierListsActive": {
        text: "Tier lists · active",
        description: "Count label: tier lists that are currently active. Keep the middle dot.",
    },
    "system.health.site.tierListsTotal": {
        text: "Tier lists · total",
        description: "Count label: every tier list on the site. Keep the middle dot.",
    },
    "system.health.site.versions": {
        text: "Published versions",
        description: "Count label: tier list versions that have been published.",
    },
    "system.health.site.placements": {
        text: "Placements",
        description: "Count label: operators placed into a tier across all tier lists.",
    },
    "system.health.game.title": {
        text: "Game data loaded",
        description: "Heading of the card counting the Arknights game data the backend has loaded.",
    },
    "system.health.game.desc": {
        text: "What the backend currently has in memory. Drift here means the asset import is out of date.",
        description: "Caption under the game data heading. 'Drift' means these counts differ from the live game.",
    },
    "system.health.game.operators": {
        text: "Operators",
        description: "Count label: playable characters in the game data.",
    },
    "system.health.game.skills": {
        text: "Skills",
        description: "Count label: operator skills in the game data.",
    },
    "system.health.game.modules": {
        text: "Modules",
        description: "Count label: operator modules (equipment upgrades) in the game data.",
    },
    "system.health.game.skins": {
        text: "Skins",
        description: "Count label: operator outfits in the game data.",
    },
    "system.health.game.stages": {
        text: "Stages",
        description: "Count label: playable levels in the game data.",
    },
    "system.health.game.zones": {
        text: "Zones",
        description: "Count label: groups of stages (chapters, events) in the game data.",
    },
    "system.health.game.enemies": {
        text: "Enemies",
        description: "Count label: enemy types in the game data.",
    },
    "system.health.statsError": {
        text: "Couldn't load these counts.",
        description: "Shown in a Health card when the site statistics request failed.",
    },

    "system.audit.source.all": {
        text: "All",
        description: "Edit history filter: show edits to both operator notes and translations.",
    },
    "system.audit.source.notes": {
        text: "Operator notes",
        description: "Edit history filter: show only edits to operator notes.",
    },
    "system.audit.source.translations": {
        text: "Translations",
        description: "Edit history filter: show only edits to translated site text.",
    },
    "system.audit.search.placeholder": {
        text: "Search edits",
        description: "Placeholder in the edit history search box. Keep it short: the box is narrow. It searches people, operators, translation keys and changed text.",
    },
    "system.audit.search.label": {
        text: "Search the edit history",
        description: "Accessible name of the edit history search box.",
    },
    "system.audit.field.label": {
        text: "Filter by field",
        description: "Accessible name of the picker that limits the edit history to one operator note field.",
    },
    "system.audit.field.all": {
        text: "All fields",
        description: "Field picker option: show edits to every field.",
    },
    "system.audit.field.summary": {
        text: "Summary",
        description: "Operator note field: the one-line summary.",
    },
    "system.audit.field.pros": {
        text: "Pros",
        description: "Operator note field: the operator's strengths.",
    },
    "system.audit.field.cons": {
        text: "Cons",
        description: "Operator note field: the operator's weaknesses.",
    },
    "system.audit.field.notes": {
        text: "Notes",
        description: "Operator note field: free-form notes.",
    },
    "system.audit.field.trivia": {
        text: "Trivia",
        description: "Operator note field: fun facts.",
    },
    "system.audit.field.text": {
        text: "Text",
        description: "Field badge on a translation edit: the translated text itself.",
    },
    "system.audit.untracked": {
        text: "Role, grant and publish changes aren't logged yet.",
        description: "Note in the edit history toolbar saying which kinds of change it does not record.",
    },
    "system.audit.th.when": {
        text: "When",
        description: "Edit history column header: when the edit was made.",
    },
    "system.audit.th.who": {
        text: "Who",
        description: "Edit history column header: who made the edit.",
    },
    "system.audit.th.what": {
        text: "What",
        description: "Edit history column header: the operator or translated string that was edited.",
    },
    "system.audit.th.field": {
        text: "Field",
        description: "Edit history column header: which field of it changed.",
    },
    "system.audit.th.change": {
        text: "Change",
        description: "Edit history column header: the kind of change and a preview of the new text.",
    },
    "system.audit.kind.added": {
        text: "added",
        description: "Badge on an edit that filled an empty field.",
    },
    "system.audit.kind.cleared": {
        text: "cleared",
        description: "Badge on an edit that emptied a field.",
    },
    "system.audit.kind.changed": {
        text: "changed",
        description: "Badge on an edit that replaced existing text.",
    },
    "system.audit.target.translation": {
        text: "{language} · {key}",
        description: "What a translation edit changed: {language} is the language's English name and {key} the identifier of the string. Keep the middle dot.",
    },
    "system.audit.actor.deleted": {
        text: "Deleted account",
        description: "Shown in the Who column when the account that made the edit no longer exists.",
    },
    "system.audit.count": {
        text: "{count, plural, one {Showing {shown} of {total} edit} other {Showing {shown} of {total} edits}}",
        description: "Footer of the edit history. {shown} is how many rows are listed and {total} how many edits exist; {count} is the total as a number, used only to pick the plural form.",
    },
    "system.audit.loadMore": {
        text: "Load more",
        description: "Button under the edit history that fetches older edits.",
    },
    "system.audit.empty.title": {
        text: "No edits yet",
        description: "Empty state title in the edit history when nothing has been logged.",
    },
    "system.audit.empty.desc": {
        text: "Operator note and translation edits show up here as soon as someone saves one.",
        description: "Empty state body in the edit history when nothing has been logged.",
    },
    "system.audit.noMatch.title": {
        text: "No edits match",
        description: "Empty state title when the search or field filter hides every loaded edit.",
    },
    "system.audit.noMatch.desc": {
        text: "Try another search or field, or load older edits.",
        description: "Empty state body when the search or field filter hides every loaded edit.",
    },
    "system.audit.error.notes": {
        text: "Couldn't load the operator note history: {message}",
        description: "Error line in the edit history; {message} explains what went wrong.",
    },
    "system.audit.error.translations": {
        text: "Couldn't load the translation history: {message}",
        description: "Error line in the edit history; {message} explains what went wrong.",
    },
    "system.audit.retry": {
        text: "Try again",
        description: "Button beside an edit history error that retries the request.",
    },

    "system.languages.title": {
        text: "Site languages",
        description: "Heading of the card listing the site's languages.",
    },
    "system.languages.desc": {
        text: "Hidden languages can be translated but don't appear in the language switcher. Only super-admins can change this.",
        description: "Caption under the Site languages heading. 'Super-admins' are the highest admin role.",
    },
    "system.languages.add": {
        text: "Add language",
        description: "Button that opens the form for adding a new site language, and the form's submit button.",
    },
    "system.languages.th.language": {
        text: "Language",
        description: "Languages table column header: the language's name.",
    },
    "system.languages.th.code": {
        text: "Code",
        description: "Languages table column header: the language tag, such as 'ja'.",
    },
    "system.languages.th.fallback": {
        text: "Falls back to",
        description: "Languages table column header: where an untranslated string is taken from instead.",
    },
    "system.languages.th.server": {
        text: "Game data from",
        description: "Languages table column header: which Arknights client supplies game text for this language.",
    },
    "system.languages.th.translated": {
        text: "Translated",
        description: "Languages table column header: how much of the site text is translated and current.",
    },
    "system.languages.th.public": {
        text: "Public",
        description: "Languages table column header: whether visitors can pick the language.",
    },
    "system.languages.th.actions": {
        text: "Actions",
        description: "Accessible name of the Languages table column holding each row's menu button.",
    },
    "system.languages.source": {
        text: "Source",
        description: "Badge on the language the site's text is written in.",
    },
    "system.languages.sourceLanguage": {
        text: "Source language",
        description: "Shown in the Translated column for the source language, which needs no translation.",
    },
    "system.languages.noFallback": {
        text: "—",
        description: "Shown in the Falls back to column for the source language, which falls back to nothing. A dash meaning 'none'.",
    },
    "system.languages.fallback.chain": {
        text: "{code}, then English",
        description: "Falls back to column: untranslated strings come from the language {code} (a language tag), then from English.",
    },
    "system.languages.fallback.english": {
        text: "English",
        description: "Falls back to column: untranslated strings come straight from English.",
    },
    "system.languages.translated.unknown": {
        text: "No data",
        description: "Shown in the Translated column when there is no progress figure for the language.",
    },
    "system.languages.server.en": {
        text: "Global (EN)",
        description: "Game region: the worldwide English client. 'EN' is its code and stays as-is.",
    },
    "system.languages.server.jp": {
        text: "Japan (JP)",
        description: "Game region: the Japanese client. 'JP' is its code and stays as-is.",
    },
    "system.languages.server.kr": {
        text: "Korea (KR)",
        description: "Game region: the Korean client. 'KR' is its code and stays as-is.",
    },
    "system.languages.server.cn": {
        text: "Mainland China (CN)",
        description: "Game region: the mainland Chinese client. 'CN' is its code and stays as-is.",
    },
    "system.languages.server.tw": {
        text: "Taiwan (TW)",
        description: "Game region: the Taiwanese client. 'TW' is its code and stays as-is.",
    },
    "system.languages.server.bili": {
        text: "Bilibili (CN)",
        description: "Game region: the Bilibili-published Chinese client. 'Bilibili' is the publisher's name and 'CN' its region code; both stay as-is.",
    },
    "system.languages.public.label": {
        text: "Show {name} in the language switcher",
        description: "Accessible name of a language's Public switch; {name} is the language's English name.",
    },
    "system.languages.menu.label": {
        text: "More actions for {name}",
        description: "Accessible name of a language row's menu button; {name} is the language's English name.",
    },
    "system.languages.menu.edit": {
        text: "Edit language",
        description: "Menu item that opens the form for changing a language's names, fallback, game data and order.",
    },
    "system.languages.empty": {
        text: "No languages configured yet.",
        description: "Shown in the Languages table when there are no languages.",
    },
    "system.languages.loadError": {
        text: "Couldn't load the languages: {message}",
        description: "Shown in the Languages card when the request failed; {message} explains what went wrong.",
    },
    "system.languages.toast.enabled.title": {
        text: "Language enabled",
        description: "Toast title after a language is made public.",
    },
    "system.languages.toast.enabled.desc": {
        text: "{name} now appears in the language switcher.",
        description: "Toast body after a language is made public; {name} is its English name.",
    },
    "system.languages.toast.hidden.title": {
        text: "Language hidden",
        description: "Toast title after a language is hidden from visitors.",
    },
    "system.languages.toast.hidden.desc": {
        text: "{name} is hidden from visitors.",
        description: "Toast body after a language is hidden; {name} is its English name.",
    },
    "system.languages.toast.toggleFailed": {
        text: "Couldn't change {name}",
        description: "Toast title when switching a language public or hidden failed; {name} is its English name.",
    },
    "system.languages.toast.created": {
        text: "Language added",
        description: "Toast title after a new language is created.",
    },
    "system.languages.toast.saved": {
        text: "Language saved",
        description: "Toast title after a language's settings are saved.",
    },
    "system.languages.toast.savedDesc": {
        text: "{name} ({code}) is up to date.",
        description: "Toast body after a language is created or saved; {name} is its English name and {code} its language tag.",
    },
    "system.languages.toast.saveFailed": {
        text: "Couldn't save the language",
        description: "Toast title when creating or saving a language failed.",
    },

    "system.languages.sheet.addTitle": {
        text: "Add a language",
        description: "Title of the side panel for creating a new site language.",
    },
    "system.languages.sheet.editTitle": {
        text: "Edit {name}",
        description: "Title of the side panel for changing a language; {name} is its English name.",
    },
    "system.languages.sheet.addDesc": {
        text: "New languages start hidden. Turn on Public in the table once enough of the site is translated.",
        description: "Line under the add-language panel title.",
    },
    "system.languages.sheet.editDesc": {
        text: "Changes reach the site as soon as you save.",
        description: "Line under the edit-language panel title.",
    },
    "system.languages.form.code": {
        text: "Code",
        description: "Label of the language tag field.",
    },
    "system.languages.form.codeHint": {
        text: "The language tag, which is also the URL prefix: two lowercase letters, optionally a region, like ja or pt-BR.",
        description: "Help under the code field. 'ja' and 'pt-BR' are example language tags and stay as-is.",
    },
    "system.languages.form.codeLocked": {
        text: "The code can't change once the language exists. Add a new language instead.",
        description: "Help under the code field when editing an existing language.",
    },
    "system.languages.form.englishName": {
        text: "English name",
        description: "Label of the field holding the language's name in English.",
    },
    "system.languages.form.englishNamePlaceholder": {
        text: "Japanese",
        description: "Example in the empty English name field: a language's name written in English.",
    },
    "system.languages.form.englishNameHint": {
        text: "How the language is listed in the admin panel.",
        description: "Help under the English name field.",
    },
    "system.languages.form.nativeName": {
        text: "Native name",
        description: "Label of the field holding the language's name in that language.",
    },
    "system.languages.form.nativeNamePlaceholder": {
        text: "日本語",
        description: "Example in the empty native name field: the word 'Japanese' written in Japanese. Substitute an example in your own language.",
    },
    "system.languages.form.nativeNameHint": {
        text: "The label visitors see in the language switcher.",
        description: "Help under the native name field.",
    },
    "system.languages.form.fallback": {
        text: "Falls back to",
        description: "Label of the picker for where an untranslated string is taken from.",
    },
    "system.languages.form.fallbackNone": {
        text: "Straight to English",
        description: "Fallback picker option: untranslated strings come straight from English.",
    },
    "system.languages.form.fallbackHint": {
        text: "Where an untranslated string looks next, before English.",
        description: "Help under the fallback picker.",
    },
    "system.languages.form.fallbackHintSource": {
        text: "The source language is the end of every fallback chain, so it has none.",
        description: "Help under the disabled fallback picker when editing the source language.",
    },
    "system.languages.form.server": {
        text: "Game data from",
        description: "Label of the picker for which Arknights client supplies game text.",
    },
    "system.languages.form.serverHint": {
        text: "Which Arknights client supplies operator, skill and stage text under this language. Translating the site doesn't translate game data; this does.",
        description: "Help under the game data picker.",
    },
    "system.languages.form.sortOrder": {
        text: "Sort order",
        description: "Label of the field that orders languages in the switcher.",
    },
    "system.languages.form.sortOrderHint": {
        text: "Lower numbers come first in the language switcher.",
        description: "Help under the sort order field.",
    },
    "system.languages.err.codeRequired": {
        text: "Enter a code. It becomes the URL prefix.",
        description: "Validation message when the language code is blank.",
    },
    "system.languages.err.codeShape": {
        text: "Use two lowercase letters, optionally a region, like ja or pt-BR. Other shapes never route.",
        description: "Validation message when the language code has the wrong shape. 'Never route' means the site won't recognise URLs using it. 'ja' and 'pt-BR' stay as-is.",
    },
    "system.languages.err.codeExists": {
        text: "{code} already exists. Edit that row instead.",
        description: "Validation message when adding a language whose code is already configured; {code} is the tag.",
    },
    "system.languages.err.englishNameRequired": {
        text: "Enter the English name.",
        description: "Validation message when the English name is blank.",
    },
    "system.languages.err.nativeNameRequired": {
        text: "Enter the native name. Visitors see it in the switcher.",
        description: "Validation message when the native name is blank.",
    },
    "system.languages.err.fallbackSource": {
        text: "The source language can't fall back to anything.",
        description: "Validation message when a fallback is picked for the source language.",
    },
    "system.languages.err.fallbackSelf": {
        text: "A language can't fall back to itself.",
        description: "Validation message when the chosen fallback is the language being edited.",
    },
    "system.languages.err.fallbackCycle": {
        text: "That choice makes the fallbacks loop back to this language. Pick another.",
        description: "Validation message when the chosen fallback would make the fallback chain loop forever.",
    },
    "system.languages.err.gamedataServer": {
        text: "Pick one of the listed game regions.",
        description: "Validation message when the game region is not one the backend knows.",
    },
    "system.languages.err.sortOrder": {
        text: "Use a whole number, 0 or more.",
        description: "Validation message when the sort order is not a non-negative whole number.",
    },
    "system.languages.form.fixFields": {
        text: "Fix the highlighted fields first.",
        description: "Shown in the language form footer when submitting with invalid fields.",
    },
    "system.languages.form.cancel": {
        text: "Cancel",
        description: "Button that closes the language form without saving.",
    },
    "system.languages.form.save": {
        text: "Save changes",
        description: "Button that saves an edited language.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
