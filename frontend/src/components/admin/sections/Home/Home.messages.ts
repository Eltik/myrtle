import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "admin";

export const messages = {
    "home.head.kicker": {
        text: "Welcome back, {nickname}",
        description: "Small label above the admin landing page title, greeting the signed-in person by nickname.",
    },
    "home.head.titleStaff": {
        text: "Inbox",
        description: "Title of the admin landing page for staff: everything waiting on an admin. Same word as the 'Inbox' section tab.",
    },
    "home.head.titleOwn": {
        text: "My work",
        description: "Title of the admin landing page for translators and tier list editors. Same words as the 'My work' section tab.",
    },
    "home.head.sub.translator": {
        text: "Only your languages are shown. Pick up where you left off.",
        description: "Line under the 'My work' title for a translator.",
    },
    "home.head.sub.tierListEditor": {
        text: "Operator notes and the tier lists you have access to.",
        description: "Line under the 'My work' title for a tier list editor, saying what the page covers.",
    },

    "home.tile.players": {
        text: "Players",
        description: "Admin landing stat tile: how many accounts the site has.",
    },
    "home.tile.players.unit": {
        text: "{count} staff",
        description: "Beside the Players number: how many of those accounts hold a staff role. {count} is a formatted number.",
    },
    "home.tile.rosters": {
        text: "Rosters synced",
        description: "Admin landing stat tile: how many accounts have synced their game roster.",
    },
    "home.tile.rosters.unit": {
        text: "{percent} of players",
        description: "Beside the Rosters synced number: that count as a share of all accounts. {percent} is a formatted percentage such as '45%'.",
    },
    "home.tile.tierLists": {
        text: "Tier lists · active",
        description: "Admin landing stat tile: how many tier lists are active.",
    },
    "home.tile.tierLists.unit": {
        text: "of {total}",
        description: "Beside the active tier list number: out of how many tier lists in total. {total} is a formatted number.",
    },
    "home.tile.placements": {
        text: "Placements",
        description: "Admin landing stat tile: how many operator placements all tier lists hold together.",
    },
    "home.tile.placements.unit": {
        text: "across all lists",
        description: "Beside the Placements number: the count covers every tier list.",
    },

    "home.queue.title": {
        text: "Needs attention",
        description: "Title of the staff card listing everything waiting on an admin.",
    },
    "home.queue.desc": {
        text: "{count, plural, one {# item} other {# items}}, most urgent first",
        description: "Under 'Needs attention': how many items are waiting, sorted by urgency.",
    },
    "home.queue.descEmpty": {
        text: "Nothing waiting",
        description: "Under 'Needs attention' when nothing is waiting.",
    },
    "home.queue.empty": {
        text: "All clear. Nothing is waiting on an admin.",
        description: "Body of the 'Needs attention' card when nothing is waiting.",
    },
    "home.queue.loading": {
        text: "Checking what needs attention…",
        description: "Body of the 'Needs attention' card while its data loads.",
    },
    "home.queue.area.translations": {
        text: "Translations",
        description: "Area label on a 'Needs attention' row about translations.",
    },
    "home.queue.area.notes": {
        text: "Operator notes",
        description: "Area label on a 'Needs attention' row about operator notes.",
    },
    "home.queue.area.people": {
        text: "People",
        description: "Area label on a 'Needs attention' row about someone's access.",
    },
    "home.queue.area.system": {
        text: "System",
        description: "Area label on a 'Needs attention' row about service health.",
    },
    "home.queue.stale.title": {
        text: "{count, plural, one {# {language} string} other {# {language} strings}} out of date",
        description: "'Needs attention' row: translated strings whose English source changed since. {language} is the language's English name, e.g. 'Japanese'.",
    },
    "home.queue.stale.sub": {
        text: "The English changed after these were translated.",
        description: "Under the out-of-date strings row, explaining what out of date means.",
    },
    "home.queue.stale.cta": {
        text: "Review",
        description: "Button on the out-of-date strings row that opens them in Translations.",
    },
    "home.queue.missing.title": {
        text: "{count, plural, one {# {language} string} other {# {language} strings}} not translated",
        description: "'Needs attention' row: strings with no translation yet. {language} is the language's English name.",
    },
    "home.queue.missing.sub": {
        text: "{language} is public, so visitors see English here.",
        description: "Under the untranslated strings row: the language is live, so the gaps show English text.",
    },
    "home.queue.missing.cta": {
        text: "Translate",
        description: "Button on the untranslated strings row that opens them in Translations.",
    },
    "home.notes.title": {
        text: "{count, plural, one {# operator} other {# operators}} with no notes",
        description: "Row or card title: how many operators have no editorial notes yet.",
    },
    "home.notes.sub.sixStars": {
        text: "Includes {count, plural, one {# six-star} other {# six-stars}}: {names}.",
        description: "Under the operators-with-no-notes row: how many of them are 6-star operators, and their names, comma separated.",
    },
    "home.notes.sub.sixStarsMore": {
        text: "Includes {count, plural, one {# six-star} other {# six-stars}}: {names} and {rest} more.",
        description: "Like 'Includes N six-stars: names.' when only the first few names are listed. {rest} is how many more are not named.",
    },
    "home.notes.sub.names": {
        text: "Includes {names}.",
        description: "On the editor's operator notes card: the 6-star operators among those with no notes, comma separated.",
    },
    "home.notes.sub.namesMore": {
        text: "Includes {names} and {rest} more.",
        description: "Like 'Includes names.' when only the first few names are listed. {rest} is how many more are not named.",
    },
    "home.notes.sub.none": {
        text: "Their pages show no guidance yet.",
        description: "Under the operators-with-no-notes row when none of them is a 6-star.",
    },
    "home.queue.notes.cta": {
        text: "Write",
        description: "Button on the operators-with-no-notes row that opens the first one in Operator notes.",
    },
    "home.queue.orphans.title": {
        text: "{count, plural, one {# language grant belongs to a deleted account} other {# language grants belong to deleted accounts}}",
        description: "'Needs attention' row: language access grants still on file for accounts that were deleted. {count} is how many grants.",
    },
    "home.queue.orphans.sub": {
        text: "The accounts are gone, so nobody can use these grants. Cleaning up removes them.",
        description: "Under the deleted-account grants row: why the grants are safe to remove.",
    },
    "home.queue.orphans.cta": {
        text: "Clean up",
        description: "Button on the deleted-account grants row. Opens a confirmation before removing the grants.",
    },
    "home.cleanup.title": {
        text: "{count, plural, one {Remove # leftover grant?} other {Remove # leftover grants?}}",
        description: "Title of the confirmation before removing language grants that belong to deleted accounts. {count} is how many grants.",
    },
    "home.cleanup.desc": {
        text: "{count, plural, one {This language grant belongs to an account that no longer exists, so no one loses access.} other {These language grants belong to accounts that no longer exist, so no one loses access.}}",
        description: "Body of the confirmation before removing language grants left by deleted accounts.",
    },
    "home.cleanup.cancel": {
        text: "Cancel",
        description: "Button that closes the leftover-grants confirmation without removing anything.",
    },
    "home.cleanup.confirm": {
        text: "{count, plural, one {Remove grant} other {Remove grants}}",
        description: "Confirm button in the leftover-grants confirmation: removes the grants of deleted accounts.",
    },
    "home.cleanup.toast.done": {
        text: "{count, plural, one {Removed # leftover grant} other {Removed # leftover grants}}",
        description: "Toast after the grants of deleted accounts are removed. {count} is how many were removed.",
    },
    "home.cleanup.toast.partial": {
        text: "{count, plural, one {Couldn't remove # grant} other {Couldn't remove # grants}}",
        description: "Error toast when some grants of deleted accounts could not be removed. {count} is how many failed.",
    },
    "home.cleanup.toast.partial.desc": {
        text: "The rest were removed. Try again from Home.",
        description: "Body of the error toast when only some leftover grants were removed.",
    },
    "home.queue.system.title": {
        text: "A service is degraded",
        description: "'Needs attention' row when the health check reports a problem.",
    },
    "home.queue.system.sub": {
        text: "Check database and cache.",
        description: "Under the degraded-service row.",
    },
    "home.queue.system.cta": {
        text: "Open",
        description: "Button on the degraded-service row that opens System health.",
    },

    "home.focus.locale.area": {
        text: "Translations · {level}",
        description: "Small label above a language card on a translator's landing page. {level} is their grant level, e.g. 'Edit'.",
    },
    "home.focus.locale.areaViewOnly": {
        text: "Translations · View only",
        description: "Small label above a language card when the translator may only look, not edit.",
    },
    "home.focus.locale.sub": {
        text: "{stale} out of date · {missing} missing",
        description: "On a language card: how many strings are out of date and how many are untranslated. Both are formatted numbers.",
    },
    "home.focus.locale.complete": {
        text: "Complete",
        description: "On a language card when every string is translated and current.",
    },
    "home.focus.locale.continue": {
        text: "Continue translating",
        description: "Button on a language card with work left.",
    },
    "home.focus.locale.review": {
        text: "Review",
        description: "Button on a complete language card the translator can edit.",
    },
    "home.focus.locale.browse": {
        text: "Browse",
        description: "Button on a language card the translator can only view.",
    },
    "home.focus.notes.area": {
        text: "Operator notes",
        description: "Small label above the operator notes card on an editor's landing page.",
    },
    "home.focus.notes.progress": {
        text: "{filled} / {total}",
        description: "Beside the notes progress bar: operators with notes out of all operators. Both are formatted numbers.",
    },
    "home.focus.notes.cta": {
        text: "Start writing",
        description: "Button on the editor's operator notes card that opens the first operator with no notes.",
    },
    "home.focus.tierList.area": {
        text: "Tier list · {level}",
        description: "Small label above a tier list card on an editor's landing page. {level} is their grant level, e.g. 'Publish'.",
    },
    "home.focus.tierList.sub": {
        text: "{tiers, plural, one {# tier} other {# tiers}} · {placements, plural, one {# placement} other {# placements}} · updated {when}",
        description: "On a tier list card: tier count, operator placement count and when it last changed (e.g. '3d ago').",
    },
    "home.focus.tierList.publish": {
        text: "Edit or publish",
        description: "Button on a tier list card when the editor may publish it.",
    },
    "home.focus.tierList.edit": {
        text: "Edit",
        description: "Button on a tier list card when the editor may edit but not publish it.",
    },
    "home.focus.empty": {
        text: "Nothing is waiting on you right now.",
        description: "Shown on a translator's or editor's landing page when they have nothing to work on.",
    },
    "home.focus.loading": {
        text: "Loading your work…",
        description: "Shown on a translator's or editor's landing page while their work loads.",
    },

    "home.access.title": {
        text: "What you can do",
        description: "Title of the card summarising the signed-in person's own access.",
    },
    "home.access.note.translator": {
        text: "Need another language, or Edit instead of View? Ask a super-admin.",
        description: "Under 'What you can do' for a translator.",
    },
    "home.access.note.tierListEditor": {
        text: "Publishing needs Publish access on that list.",
        description: "Under 'What you can do' for a tier list editor.",
    },
    "home.access.note.tierListAdmin": {
        text: "Roles and language grants are set by a super-admin.",
        description: "Under 'What you can do' for a tier list admin.",
    },
    "home.access.note.superAdmin": {
        text: "Change anyone’s role and grants from People.",
        description: "Under 'What you can do' for a super-admin.",
    },
    "home.access.everything": {
        text: "Everything",
        description: "The only 'What you can do' row for a super-admin.",
    },
    "home.access.siteRole": {
        text: "Site role",
        description: "'What you can do' row: the person's site-wide role, shown as a badge.",
    },
    "home.access.everyList": {
        text: "Every tier list and note",
        description: "'What you can do' row for a tier list admin.",
    },
    "home.access.notes": {
        text: "Operator notes",
        description: "'What you can do' row for a tier list editor: they may edit operator notes.",
    },

    "home.recent.titleStaff": {
        text: "Recent changes",
        description: "Title of the staff card listing the latest saved edits by anyone.",
    },
    "home.recent.titleOwn": {
        text: "Your recent changes",
        description: "Title of the card listing the signed-in person's latest saved edits.",
    },
    "home.recent.empty": {
        text: "No saved changes yet.",
        description: "Shown in the recent changes card when there are none.",
    },
    "home.recent.deletedAccount": {
        text: "Deleted account",
        description: "Stands in for the name of whoever made a recent change when their account has since been deleted. Fills {actor} in the recent-change lines.",
    },
    "home.recent.note": {
        text: "{actor} · {operator} · {field}",
        description: "Recent change row: who edited which operator's note, and which field (e.g. 'Pros').",
    },
    "home.recent.noteOwn": {
        text: "{operator} · {field}",
        description: "Your recent change row: which operator's note you edited, and which field.",
    },
    "home.recent.translation": {
        text: "{actor} · {language} · {key}",
        description: "Recent change row: who edited a translation, in which language, and the message key.",
    },
    "home.recent.translationOwn": {
        text: "{language} · {key}",
        description: "Your recent change row: the language and message key of a translation you edited.",
    },
    "home.field.summary": {
        text: "Summary",
        description: "Operator note field name.",
    },
    "home.field.pros": {
        text: "Pros",
        description: "Operator note field name.",
    },
    "home.field.cons": {
        text: "Cons",
        description: "Operator note field name.",
    },
    "home.field.notes": {
        text: "Notes",
        description: "Operator note field name: the long-form guidance.",
    },
    "home.field.trivia": {
        text: "Trivia",
        description: "Operator note field name.",
    },
    "home.field.tags": {
        text: "Tags",
        description: "Operator note field name.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
