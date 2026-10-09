import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "admin";

export const messages = {
    "users.head.kicker": {
        text: "Manage",
        description: "Small label above the People page title: the admin area this page belongs to.",
    },
    "users.head.title": {
        text: "People",
        description: "Title of the admin page listing players and their access.",
    },
    "users.head.sub": {
        text: "Players with synced profiles. Search by nickname or UID, then open someone to see and change everything they can access.",
        description: "Line under the People page title explaining what the page is for.",
    },

    "users.search.placeholder": {
        text: "Nickname or UID",
        description: "Placeholder of the People search box. A UID is the player's numeric in-game ID.",
    },
    "users.search.label": {
        text: "Search people",
        description: "Accessible name of the People search box.",
    },
    "users.filter.label": {
        text: "Filter by role",
        description: "Accessible name of the row of role filter tabs on the People page.",
    },
    "users.filter.all": {
        text: "Everyone",
        description: "People role filter tab: show every account.",
    },
    "users.filter.staff": {
        text: "Staff",
        description: "People role filter tab: super-admins, tier list admins and tier list editors.",
    },
    "users.filter.translators": {
        text: "Translators",
        description: "People role filter tab: accounts with the Translator role.",
    },
    "users.filter.players": {
        text: "Players",
        description: "People role filter tab: ordinary accounts with no staff role.",
    },
    "users.server.label": {
        text: "Filter by server",
        description: "Accessible name of the row of game-server filter tabs on the People page.",
    },
    "users.server.all": {
        text: "All",
        description: "People server filter tab: every game server. The other tabs are server codes (EN, JP, KR, CN).",
    },
    "users.count": {
        text: "{shown} of {total} shown",
        description: "Right of the People toolbar: how many rows are loaded out of every account the filters match. Both are formatted numbers.",
    },
    "users.more": {
        text: "Show more",
        description: "Button under the People table that loads the next page of accounts.",
    },
    "users.loading": {
        text: "Loading people…",
        description: "Shown in the People table while the first page loads.",
    },
    "users.error": {
        text: "Couldn't load people. {reason}",
        description: "Shown in the People table when the list fails to load. {reason} is a sentence explaining the failure.",
    },
    "users.empty": {
        text: "No one matches that search.",
        description: "Shown in the People table when no account matches the search and filters.",
    },

    "users.col.player": {
        text: "Player",
        description: "People table column header: the account's avatar, nickname and UID.",
    },
    "users.col.server": {
        text: "Server",
        description: "People table column header: the game server code (EN, JP, …).",
    },
    "users.col.level": {
        text: "Lv",
        description: "People table column header, abbreviated: the player's Doctor level in the game.",
    },
    "users.col.score": {
        text: "Score",
        description: "People table column header: the account's total roster score on this site.",
    },
    "users.col.grade": {
        text: "Grade",
        description: "People table column header: the letter grade derived from the score.",
    },
    "users.col.role": {
        text: "Role",
        description: "People table column header: the account's site-wide role.",
    },
    "users.col.canEdit": {
        text: "Can edit",
        description: "People table column header: a summary of what the account can change in the admin panel.",
    },
    "users.deletedAccount": {
        text: "Deleted account",
        description: "Stands in for a person's name when their account has been deleted but something (such as a language grant) still refers to it.",
    },
    "users.uid": {
        text: "UID {uid}",
        description: "Under a player's nickname: their in-game numeric ID.",
    },

    "users.access.everything": {
        text: "Everything",
        description: "'Can edit' cell for a super-admin.",
    },
    "users.access.allLists": {
        text: "All tier lists, notes",
        description: "'Can edit' cell for a tier list admin: every tier list plus operator notes.",
    },
    "users.access.editor": {
        text: "Operator notes, {lists}",
        description: "'Can edit' cell for a tier list editor: operator notes plus the titles of the tier lists granted to them, comma separated.",
    },
    "users.access.editorNoLists": {
        text: "Operator notes",
        description: "'Can edit' cell for a tier list editor with no tier lists granted: operator notes only. This is a normal setup, not a problem.",
    },
    "users.access.withLanguages": {
        text: "{access}, {languages}",
        description: "'Can edit' cell when the account also holds language grants. {access} is what its role lets it edit (e.g. 'All tier lists, notes'); {languages} is a comma-separated list of language names it can edit.",
    },
    "users.access.translatorNone": {
        text: "Nothing yet",
        description: "'Can edit' cell for a translator with no Edit grant on any language.",
    },
    "users.access.warned": {
        text: "⚠ {access}",
        description: "'Can edit' cell when the account's grants contradict its role: a warning sign before the usual summary.",
    },

    "users.warn.translatorNoEdit": {
        text: "{name} has the Translator role but no Edit grant on any language, so they can't change anything yet.",
        description: "Warning in a person's detail panel: a translator who cannot edit any language.",
    },

    "users.role.desc.user": {
        text: "Public site, plus any language granted below.",
        description: "Explanation of the Player role, under the role buttons in a person's detail panel.",
    },
    "users.role.desc.translator": {
        text: "Can open Translations, but only edits the languages granted below.",
        description: "Explanation of the Translator role, under the role buttons in a person's detail panel.",
    },
    "users.role.desc.tierListEditor": {
        text: "Edits operator notes and the tier lists granted below.",
        description: "Explanation of the Tier list editor role, under the role buttons in a person's detail panel.",
    },
    "users.role.desc.tierListAdmin": {
        text: "Edits and publishes every tier list and operator note. No per-list grants needed.",
        description: "Explanation of the Tier list admin role, under the role buttons in a person's detail panel.",
    },
    "users.role.desc.superAdmin": {
        text: "Full access, including roles, grants and site languages.",
        description: "Explanation of the Super-admin role, under the role buttons in a person's detail panel.",
    },

    "users.level.view": {
        text: "View",
        description: "Grant level: may look but not change. Levels in order: View, Edit, Publish, Admin.",
    },
    "users.level.edit": {
        text: "Edit",
        description: "Grant level: may change content. Levels in order: View, Edit, Publish, Admin.",
    },
    "users.level.publish": {
        text: "Publish",
        description: "Grant level: may change and publish a tier list. Levels in order: View, Edit, Publish, Admin.",
    },
    "users.level.admin": {
        text: "Admin",
        description: "Grant level: full control, including other people's access. Levels in order: View, Edit, Publish, Admin.",
    },

    "users.sheet.meta": {
        text: "UID {uid} · {server} · Lv {level}",
        description: "Under the nickname in a person's detail panel: UID, game server code and Doctor level.",
    },
    "users.sheet.metaNoLevel": {
        text: "UID {uid} · {server}",
        description: "Under the nickname in a person's detail panel when the level is unknown: UID and game server code.",
    },
    "users.sheet.grade": {
        text: "Grade {grade}",
        description: "Badge in a person's detail panel showing their score grade letter.",
    },
    "users.sheet.publicProfile": {
        text: "Public profile",
        description: "Badge in a person's detail panel: their profile is visible to everyone.",
    },
    "users.sheet.privateProfile": {
        text: "Private profile",
        description: "Badge in a person's detail panel: their profile is hidden from other visitors.",
    },
    "users.sheet.score.total": {
        text: "Total score",
        description: "Label in a person's detail panel: their total roster score.",
    },
    "users.sheet.score.operators": {
        text: "Operators",
        description: "Label in a person's detail panel: how many operators their roster holds.",
    },
    "users.sheet.score.items": {
        text: "Items",
        description: "Label in a person's detail panel: how many item kinds their depot holds.",
    },
    "users.sheet.score.skins": {
        text: "Skins",
        description: "Label in a person's detail panel: how many outfits they own.",
    },
    "users.sheet.section.role": {
        text: "Site-wide role",
        description: "Section label in a person's detail panel above the role buttons. Rendered in capitals.",
    },
    "users.sheet.roleSelf": {
        text: "You can't change your own role.",
        description: "Under the role buttons when the open person is the signed-in admin.",
    },
    "users.sheet.section.tierLists": {
        text: "Tier lists",
        description: "Section label in a person's detail panel above their tier-list grants. Rendered in capitals.",
    },
    "users.sheet.section.locales": {
        text: "Translation languages",
        description: "Section label in a person's detail panel above their language grants. Rendered in capitals.",
    },
    "users.sheet.grant": {
        text: "+ Grant",
        description: "Button in a person's detail panel that opens the picker for a new grant.",
    },
    "users.sheet.tl.viaRole": {
        text: "Edits and publishes every tier list through their role. No grants needed.",
        description: "Shown under Tier lists for a tier list admin or super-admin.",
    },
    "users.sheet.tl.empty": {
        text: "No per-list grants.",
        description: "Shown under Tier lists when the person holds no tier-list grants.",
    },
    "users.sheet.loc.viaRole": {
        text: "Can edit every language as a super-admin.",
        description: "Shown under Translation languages for a super-admin.",
    },
    "users.sheet.loc.empty": {
        text: "No language grants.",
        description: "Shown under Translation languages when the person holds no language grants.",
    },
    "users.sheet.grantedBy": {
        text: "Granted by {name} · {when}",
        description: "Under a grant: who created it and how long ago (e.g. '3d ago').",
    },
    "users.sheet.grantedAt": {
        text: "Granted {when}",
        description: "Under a grant whose creator is unknown: how long ago it was created (e.g. '3d ago').",
    },
    "users.sheet.revoke": {
        text: "Revoke",
        description: "Button beside a grant that removes it.",
    },
    "users.sheet.picker.level": {
        text: "Level",
        description: "Label before the grant level buttons (View, Edit, Publish, Admin) in the new-grant picker.",
    },
    "users.sheet.picker.on": {
        text: "Grant {level} on:",
        description: "Label above the list of tier lists or languages in the new-grant picker. {level} is a grant level such as 'Edit'.",
    },
    "users.sheet.picker.noLists": {
        text: "Every official tier list is already granted.",
        description: "In the tier-list grant picker when there is no list left to grant.",
    },
    "users.sheet.picker.noLocales": {
        text: "Every language is already granted.",
        description: "In the language grant picker when there is no language left to grant.",
    },
    "users.sheet.picker.loading": {
        text: "Loading tier lists…",
        description: "In the tier-list grant picker while the official tier lists load.",
    },
    "users.sheet.loadingGrants": {
        text: "Loading grants…",
        description: "Shown in a person's detail panel while their grants load.",
    },
    "users.sheet.notFound.title": {
        text: "Person not found",
        description: "Title of the detail panel when the account in the link is not in the loaded list.",
    },
    "users.sheet.deleted.body": {
        text: "This account was deleted. Its leftover language grants can be removed from Home.",
        description: "Body of the detail panel opened on a deleted account that still holds language grants. 'Home' is the admin panel's first page.",
    },
    "users.sheet.notFound.body": {
        text: "This account isn't in the list. Search for them by nickname or UID to open their details.",
        description: "Body of the detail panel when the account in the link is not in the loaded list.",
    },
    "users.sheet.footer": {
        text: "Role changes apply the next time their session refreshes.",
        description: "Footer note in a person's detail panel.",
    },
    "users.sheet.profile": {
        text: "Public profile",
        description: "Footer button in a person's detail panel that opens their public profile page in a new tab.",
    },

    "users.toast.roleUpdated": {
        text: "Role updated",
        description: "Toast title after an admin changes someone's role.",
    },
    "users.toast.roleUpdated.desc": {
        text: "{name} is now {role}.",
        description: "Toast body after a role change. {role} is the new role's name, e.g. 'Translator'.",
    },
    "users.toast.roleFailed": {
        text: "Couldn't change the role",
        description: "Error toast title when a role change fails.",
    },
    "users.toast.grantCreated": {
        text: "Grant created",
        description: "Toast title after a new tier-list or language grant is saved.",
    },
    "users.toast.tlGranted.desc": {
        text: "{name} can now {level, select, view {view} edit {edit} publish {publish} other {administer}} {title}.",
        description: "Toast body after a tier-list grant. {title} is the tier list's title; the verb follows the chosen level.",
    },
    "users.toast.locGranted.desc": {
        text: "{name} can now {level, select, view {view} edit {edit} publish {publish} other {administer}} {language}.",
        description: "Toast body after a language grant. {language} is the language's English name; the verb follows the chosen level.",
    },
    "users.toast.grantFailed": {
        text: "Couldn't create the grant",
        description: "Error toast title when saving a grant fails.",
    },
    "users.toast.tlRevoked": {
        text: "Access revoked",
        description: "Toast title after a tier-list grant is removed.",
    },
    "users.toast.locRevoked": {
        text: "Grant revoked",
        description: "Toast title after a language grant is removed.",
    },
    "users.toast.revoked.desc": {
        text: "{name} on {target}",
        description: "Toast body after a grant is removed: whose grant, on which tier list or language.",
    },
    "users.toast.revokeFailed": {
        text: "Couldn't revoke the grant",
        description: "Error toast title when removing a grant fails.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
