import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "admin";

export const messages = {
    "tierLists.head.kicker": {
        text: "Manage",
        description: "Small label above the tier lists admin page title: the admin area this page belongs to.",
    },
    "tierLists.head.titleStaff": {
        text: "Official tier lists",
        description: "Title of the tier lists admin page for staff, who manage the site's official tier lists.",
    },
    "tierLists.head.titleOwn": {
        text: "My tier lists",
        description: "Title of the tier lists admin page for non-staff: only the lists they have been granted.",
    },
    "tierLists.head.subStaff": {
        text: "Lists marked official appear in the site’s Official rail. Publishing saves a new version that visitors see.",
        description: "Line under the 'Official tier lists' title. The 'Official rail' is the row of official lists on the public tier lists page.",
    },
    "tierLists.head.subOwn": {
        text: "The tier lists you’ve been granted. Your level on each list decides whether you can publish.",
        description: "Line under the 'My tier lists' title for a non-staff editor.",
    },
    "tierLists.head.newList": {
        text: "New official list",
        description: "Button in the page heading (staff only) that opens a dialog to create a new official tier list.",
    },

    "tierLists.tabs.lists": {
        text: "Lists",
        description: "Tab showing the table of tier lists.",
    },
    "tierLists.tabs.access": {
        text: "Access",
        description: "Tab showing who holds a per-list grant on which tier list. A count of grants follows it.",
    },

    "tierLists.lists.searchLabel": {
        text: "Search tier lists",
        description: "Accessible name of the search box above the tier lists table.",
    },
    "tierLists.lists.searchPlaceholder": {
        text: "Title, slug or flair",
        description: "Placeholder of the search box above the tier lists table: what it matches. A slug is the list's URL name.",
    },
    "tierLists.lists.filter.all": {
        text: "All · {count}",
        description: "Filter tab showing every list. {count} is how many lists that is, already formatted.",
    },
    "tierLists.lists.filter.hot": {
        text: "Trending · {count}",
        description: "Filter tab showing only lists that are trending right now. {count} is how many, already formatted.",
    },
    "tierLists.lists.type.label": {
        text: "List type",
        description: "Accessible name of the dropdown that switches the table between official lists, community lists or both.",
    },
    "tierLists.lists.type.official": {
        text: "Official lists",
        description: "Option of the list-type dropdown: lists published by the site's staff.",
    },
    "tierLists.lists.type.community": {
        text: "Community lists",
        description: "Option of the list-type dropdown: lists made by players.",
    },
    "tierLists.lists.type.all": {
        text: "All lists",
        description: "Option of the list-type dropdown: official and community lists together.",
    },
    "tierLists.lists.th.title": {
        text: "Title",
        description: "Column heading of the tier lists table: the list's title, slug and author.",
    },
    "tierLists.lists.th.flair": {
        text: "Flair",
        description: "Column heading: the coloured category tag on the list (for example Endgame or Beginner).",
    },
    "tierLists.lists.th.trending": {
        text: "Trending",
        description: "Column heading: whether the list is trending right now.",
    },
    "tierLists.lists.th.tiers": {
        text: "Tiers",
        description: "Column heading: how many tiers (rows such as S, A, B) the list has.",
    },
    "tierLists.lists.th.placements": {
        text: "Placements",
        description: "Column heading: how many operators are placed in the list's tiers.",
    },
    "tierLists.lists.th.views": {
        text: "Views 24h",
        description: "Column heading: how many times the list was viewed in the last 24 hours.",
    },
    "tierLists.lists.th.updated": {
        text: "Updated",
        description: "Column heading: when the list was last changed.",
    },
    "tierLists.lists.th.actions": {
        text: "Actions",
        description: "Column heading of the action buttons, read by screen readers only.",
    },
    "tierLists.lists.trending": {
        text: "Trending",
        description: "Badge on a row whose list is trending right now.",
    },
    "tierLists.lists.community": {
        text: "Community",
        description: "Small badge next to a list title marking it as a player-made community list rather than an official one.",
    },
    "tierLists.lists.unlisted": {
        text: "Not in the public listing",
        description: "Shown under a list title instead of its author when the list is hidden from the public tier lists page, so no stats are available.",
    },
    "tierLists.lists.edit": {
        text: "Edit",
        description: "Row button that opens the tier list editor.",
    },
    "tierLists.lists.view": {
        text: "View",
        description: "Row button that opens the public page of a list the user may only view, not edit.",
    },
    "tierLists.lists.publish": {
        text: "Publish",
        description: "Row button that opens the inline form for publishing a new version of the list.",
    },
    "tierLists.lists.more": {
        text: "More actions for {title}",
        description: "Accessible name of the ⋯ button on a row that opens its menu. {title} is the list's title.",
    },
    "tierLists.lists.empty.noMatch": {
        text: "Nothing matches.",
        description: "Shown in the table when the search or filter hides every list.",
    },
    "tierLists.lists.empty.noAccess": {
        text: "You don’t have access to any tier lists yet.",
        description: "Shown in the table when the user holds no grant on any list.",
    },
    "tierLists.lists.empty.noLists": {
        text: "No lists of this type yet.",
        description: "Shown to staff when there are no lists of the chosen type (official or community) at all.",
    },
    "tierLists.lists.loadError": {
        text: "Couldn’t load the tier lists. Try again in a moment.",
        description: "Shown in place of the table when loading the lists failed.",
    },

    "tierLists.menu.viewPublic": {
        text: "View public page",
        description: "Row menu item: open the list's public page in a new tab.",
    },
    "tierLists.menu.changeFlair": {
        text: "Change flair",
        description: "Row menu item: open a dialog to change the list's flair (coloured category tag).",
    },
    "tierLists.menu.delete": {
        text: "Delete list",
        description: "Row menu item: delete the tier list after a confirmation.",
    },

    "tierLists.publish.prompt": {
        text: "Publish v{version} of {title}. What changed?",
        description: "Lead-in of the inline publish form. {version} is the number the new version will get, {title} the list's title.",
    },
    "tierLists.publish.promptPending": {
        text: "Publish a new version of {title}. What changed?",
        description: "Lead-in of the inline publish form while the next version number is still loading. {title} is the list's title.",
    },
    "tierLists.publish.changelogLabel": {
        text: "What changed",
        description: "Accessible name of the changelog field in the inline publish form.",
    },
    "tierLists.publish.changelogPlaceholder": {
        text: "Optional. Shown in version history.",
        description: "Placeholder of the changelog field: a short note on what this version changes, shown in the list's version history.",
    },
    "tierLists.publish.confirm": {
        text: "Publish now",
        description: "Button that publishes the new version.",
    },
    "tierLists.publish.toast": {
        text: "Version published",
        description: "Title of the toast shown after a version is published.",
    },
    "tierLists.publish.toastDesc": {
        text: "{title} v{version}",
        description: "Body of the toast after publishing. {title} is the list's title, {version} the new version number.",
    },
    "tierLists.publish.failed": {
        text: "Couldn’t publish",
        description: "Title of the error toast when publishing a version failed.",
    },

    "tierLists.level.view": {
        text: "View",
        description: "Grant level: may only view the list. Lowest of View, Edit, Publish, Admin.",
    },
    "tierLists.level.edit": {
        text: "Edit",
        description: "Grant level: may edit the list but not publish it.",
    },
    "tierLists.level.publish": {
        text: "Publish",
        description: "Grant level: may edit the list and publish new versions.",
    },
    "tierLists.level.admin": {
        text: "Admin",
        description: "Grant level: full control of the list, including deleting it. Highest level.",
    },

    "tierLists.access.title": {
        text: "Per-list grants",
        description: "Title of the card listing every per-list tier list grant.",
    },
    "tierLists.access.desc": {
        text: "View → Edit → Publish → Admin. Tier list admins and super-admins already have access to every list.",
        description: "Line under 'Per-list grants': the levels from lowest to highest, and that admins need no grant.",
    },
    "tierLists.access.grant": {
        text: "Grant access",
        description: "Button that opens the dialog for granting a player access to a tier list.",
    },
    "tierLists.access.th.player": {
        text: "Player",
        description: "Column heading of the grants table: who holds the grant.",
    },
    "tierLists.access.th.list": {
        text: "Tier list",
        description: "Column heading: the list the grant is on.",
    },
    "tierLists.access.th.level": {
        text: "Level",
        description: "Column heading: the grant's level (View, Edit, Publish or Admin).",
    },
    "tierLists.access.th.granted": {
        text: "Granted",
        description: "Column heading: who created the grant and when.",
    },
    "tierLists.access.th.actions": {
        text: "Actions",
        description: "Column heading of the row buttons, read by screen readers only.",
    },
    "tierLists.access.uid": {
        text: "UID {uid}",
        description: "The player's in-game user ID under their nickname. {uid} is the number.",
    },
    "tierLists.access.unnamed": {
        text: "Unnamed player",
        description: "Shown in place of a nickname for an account that has none.",
    },
    "tierLists.access.grantedBy": {
        text: "{name} · {when}",
        description: "Who created the grant and when. {name} is a nickname, {when} a relative time like '2d ago'.",
    },
    "tierLists.access.unknownGranter": {
        text: "Unknown",
        description: "Stands in for the granter's name when the account that created the grant no longer exists.",
    },
    "tierLists.access.changeLevel": {
        text: "Change {name}’s level on {title}",
        description: "Accessible name of the level badge button that opens the level menu. {name} is the player, {title} the list.",
    },
    "tierLists.access.profile": {
        text: "Profile",
        description: "Row button that opens this player in the People section.",
    },
    "tierLists.access.revoke": {
        text: "Revoke",
        description: "Row button that removes this grant.",
    },
    "tierLists.access.empty": {
        text: "No one holds a per-list grant yet.",
        description: "Shown in the grants table when there are no grants.",
    },
    "tierLists.access.loadError": {
        text: "Couldn’t load the grants. Try again in a moment.",
        description: "Shown in place of the grants table when loading failed.",
    },
    "tierLists.access.toast.revoked": {
        text: "Access revoked",
        description: "Title of the toast after a grant is removed.",
    },
    "tierLists.access.toast.revokedDesc": {
        text: "{name} on {title}",
        description: "Body of the toast after revoking. {name} is the player, {title} the list.",
    },
    "tierLists.access.toast.revokeFailed": {
        text: "Couldn’t revoke access",
        description: "Title of the error toast when removing a grant failed.",
    },
    "tierLists.access.toast.levelChanged": {
        text: "Level changed",
        description: "Title of the toast after a grant's level is changed.",
    },
    "tierLists.access.toast.levelDesc": {
        text: "{name} now has {level} on {title}.",
        description: "Body of the toast after granting or changing a level. {name} is the player, {level} a level name like Edit, {title} the list.",
    },
    "tierLists.access.toast.levelFailed": {
        text: "Couldn’t change the level",
        description: "Title of the error toast when changing a grant's level failed.",
    },
    "tierLists.access.toast.granted": {
        text: "Grant created",
        description: "Title of the toast after a new grant is created.",
    },
    "tierLists.access.toast.grantFailed": {
        text: "Couldn’t create the grant",
        description: "Title of the error toast when creating a grant failed.",
    },

    "tierLists.grant.title": {
        text: "Grant access to a tier list",
        description: "Title of the dialog for granting a player access to one tier list.",
    },
    "tierLists.grant.desc": {
        text: "The grant only takes effect while the player has a staff role.",
        description: "Line under the grant dialog title: a plain player's grants do nothing until they get a staff role such as Tier list editor.",
    },
    "tierLists.grant.player": {
        text: "Player",
        description: "Label of the player search field in the grant dialog.",
    },
    "tierLists.grant.playerPlaceholder": {
        text: "Nickname or UID",
        description: "Placeholder of the player search field in the grant dialog.",
    },
    "tierLists.grant.noMatches": {
        text: "No players match.",
        description: "Shown under the player search when nobody matches.",
    },
    "tierLists.grant.list": {
        text: "Tier list",
        description: "Label of the list picker in the grant dialog.",
    },
    "tierLists.grant.listPlaceholder": {
        text: "Choose a list",
        description: "Placeholder of the list picker in the grant dialog before a list is chosen.",
    },
    "tierLists.grant.level": {
        text: "Level",
        description: "Label of the level picker in the grant dialog.",
    },
    "tierLists.grant.submit": {
        text: "Grant {level}",
        description: "Submit button of the grant dialog. {level} is the chosen level name, like Edit.",
    },

    "tierLists.new.title": {
        text: "New official list",
        description: "Title of the dialog that creates a new official tier list.",
    },
    "tierLists.new.desc": {
        text: "Creates an empty draft. Add tiers and flair in the editor.",
        description: "Line under the new-list dialog title.",
    },
    "tierLists.new.nameLabel": {
        text: "Title",
        description: "Label of the title field in the new-list dialog.",
    },
    "tierLists.new.namePlaceholder": {
        text: "Endgame DPS rankings",
        description: "Example title shown as the placeholder of the title field in the new-list dialog.",
    },
    "tierLists.new.descLabel": {
        text: "Description",
        description: "Label of the description field in the new-list dialog.",
    },
    "tierLists.new.descPlaceholder": {
        text: "Optional. Shown under the title on the list’s page.",
        description: "Placeholder of the description field in the new-list dialog.",
    },
    "tierLists.new.submit": {
        text: "Create and open editor",
        description: "Submit button of the new-list dialog: creates the list, then opens it in the editor.",
    },
    "tierLists.new.toast": {
        text: "Draft created",
        description: "Title of the toast after a new official list is created.",
    },
    "tierLists.new.toastDesc": {
        text: "Add tiers and flair in the editor.",
        description: "Body of the toast after creating a list.",
    },
    "tierLists.new.failed": {
        text: "Couldn’t create the list",
        description: "Title of the error toast when creating a list failed.",
    },

    "tierLists.flair.title": {
        text: "Change flair",
        description: "Title of the dialog for changing a list's flair (coloured category tag).",
    },
    "tierLists.flair.desc": {
        text: "The flair shows as a coloured tag on {title} across the site.",
        description: "Line under the flair dialog title. {title} is the list's title.",
    },
    "tierLists.flair.none": {
        text: "No flair",
        description: "Flair option that removes the list's flair.",
    },
    "tierLists.flair.toast": {
        text: "Flair updated",
        description: "Title of the toast after a list's flair is changed.",
    },
    "tierLists.flair.failed": {
        text: "Couldn’t change the flair",
        description: "Title of the error toast when changing the flair failed.",
    },

    "tierLists.delete.title": {
        text: "Delete {title}?",
        description: "Title of the confirmation dialog for deleting a list. {title} is the list's title.",
    },
    "tierLists.delete.body": {
        text: "The list, its tiers and every published version are removed for everyone. This can’t be undone.",
        description: "Warning in the delete confirmation dialog.",
    },
    "tierLists.delete.submit": {
        text: "Delete list",
        description: "Confirm button of the delete dialog.",
    },
    "tierLists.delete.toast": {
        text: "List deleted",
        description: "Title of the toast after a list is deleted. The body is the list's title.",
    },
    "tierLists.delete.failed": {
        text: "Couldn’t delete the list",
        description: "Title of the error toast when deleting a list failed.",
    },

    "tierLists.cancel": {
        text: "Cancel",
        description: "Button that closes a dialog or the inline publish form without saving.",
    },
    "tierLists.save": {
        text: "Save",
        description: "Button that saves the flair dialog.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
