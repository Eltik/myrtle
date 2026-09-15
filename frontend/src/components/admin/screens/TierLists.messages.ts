import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "admin";

export const messages = {
    "perms.kicker": {
        text: "Manage",
        description: "Eyebrow over the Tier Lists page title, naming the admin section it belongs to. Rendered uppercase.",
    },
    "perms.title": {
        text: "Tier Lists",
        description: "Title of the admin screen that lists every tier list and its permission grants.",
    },
    "perms.sub": {
        text: "Every tier list on the platform. Pick one to manage its {ladder} permission ladder, or use the row menu for direct edits.",
        description: "Sentence under the Tier Lists title. {ladder} is the four rungs of the permission ladder, shown in bold.",
    },
    "perms.sub.ladder": {
        text: "View → Edit → Publish → Admin",
        description: "The four rungs of the permission ladder, in increasing order, shown in bold. Keep the arrows; the rung names match the buttons on each grant.",
    },
    "perms.list.title": {
        text: "Tier lists",
        description: "Heading of the left-hand card listing every tier list.",
    },
    "perms.list.counts": {
        text: "{total} total · {official} official",
        description: "Caption under the Tier lists heading: how many lists there are and how many of those are site-run. Keep the middle dot.",
    },
    "perms.searchPlaceholder": {
        text: "Search slug or owner…",
        description: "Prompt in the tier-list search box. A 'slug' is the list's URL name. Keep the ellipsis character.",
    },
    "perms.list.empty": {
        text: "No tier lists match.",
        description: "Empty state in the tier-list column when the search matched nothing.",
    },
    "perms.pickOne": {
        text: "Pick a tier list on the left.",
        description: "Placeholder in the detail pane before a tier list has been selected.",
    },
    "perms.row.meta": {
        text: "{author} · {count, plural, one {# tier} other {# tiers}}",
        description: "Second line of a tier-list row: who owns it and how many tiers it has. Keep the middle dot.",
    },
    "perms.badge.official": {
        text: "official",
        description: "Badge on a tier list that the site itself publishes.",
    },
    "perms.rowActions": {
        text: "Row actions",
        description: "Accessible name of the three-dots button that opens a tier list's action menu.",
    },
    "perms.menu.viewPublic": {
        text: "View public page",
        description: "Menu item that opens the tier list's public page in a new tab.",
    },
    "perms.menu.openEditor": {
        text: "Open editor",
        description: "Menu item that opens the tier list in the editor.",
    },
    "perms.menu.changeFlair": {
        text: "Change flair",
        description: "Menu item that opens the flair picker. A 'flair' is the coloured tag shown on a list.",
    },
    "perms.menu.delete": {
        text: "Delete tier list",
        description: "Destructive menu item that opens the delete confirmation.",
    },
    "perms.detail.meta": {
        text: "{title} · owner {owner} · {count, plural, one {# grant} other {# grants}}",
        description: "Caption over a tier list's permission table: its title, who owns it, and how many people hold a grant on it. Keep the middle dots.",
    },
    "perms.grantAccess": {
        text: "Grant access",
        description: "Button that opens the dialog for giving someone a permission on this tier list.",
    },
    "perms.loadError": {
        text: "Couldn't load permissions for {slug}.",
        description: "Error line in the detail pane. {slug} is the tier list's URL name, shown in monospace.",
    },
    "perms.noGrants": {
        text: "No grants yet - only the owner has access.",
        description: "Empty state in a tier list's permission table. Keep the hyphen.",
    },
    "perms.th.doctor": {
        text: "Doctor",
        description: "Permission table column header for who holds the grant. 'Doctor' is what Arknights calls the player. Rendered uppercase.",
    },
    "perms.th.level": {
        text: "Level",
        description: "Permission table column header for which rung of the ladder the grant is. Rendered uppercase.",
    },
    "perms.th.granted": {
        text: "Granted",
        description: "Permission table column header for when the grant was made. Rendered uppercase.",
    },
    "perms.th.grantedBy": {
        text: "Granted by",
        description: "Permission table column header for the admin who made the grant. Rendered uppercase.",
    },
    "perms.revoke": {
        text: "Revoke",
        description: "Button that removes one permission grant. A verb.",
    },
    "perms.ownerNote": {
        text: "Owner has implicit Admin and cannot be revoked.",
        description: "Footer note under a tier list's permission table. 'Admin' is the top rung of the permission ladder.",
    },
    "perms.toast.updated": {
        text: "Permission updated",
        description: "Toast title after a grant's level was changed.",
    },
    "perms.toast.updated.desc": {
        text: "Updated grant on /{slug}.",
        description: "Toast body after a grant's level was changed; {slug} is the tier list's URL name. Keep the leading slash.",
    },
    "perms.toast.updateFailed": {
        text: "Failed to update permission",
        description: "Toast title after a grant change was rejected.",
    },
    "perms.toast.revoked": {
        text: "Permission revoked",
        description: "Toast title after a grant was removed.",
    },
    "perms.toast.revoked.desc": {
        text: "Removed grant on /{slug}.",
        description: "Toast body after a grant was removed; {slug} is the tier list's URL name. Keep the leading slash.",
    },
    "perms.toast.revokeFailed": {
        text: "Failed to revoke",
        description: "Toast title after removing a grant was rejected.",
    },
    "perms.grant.kicker": {
        text: "Manage · permissions",
        description: "Eyebrow at the top of the grant dialog. Keep the middle dot. Rendered uppercase.",
    },
    "perms.grant.title": {
        text: "Grant access on /{slug}",
        description: "Heading of the grant dialog; {slug} is the tier list's URL name. Keep the leading slash.",
    },
    "perms.grant.findDoctor": {
        text: "Find a Doctor",
        description: "Label over the account search in the grant dialog. 'Doctor' is what Arknights calls the player.",
    },
    "perms.grant.searchPlaceholder": {
        text: "Search by nickname…",
        description: "Prompt in the account search box of the grant dialog. Keep the ellipsis character.",
    },
    "perms.grant.noMatches": {
        text: "No matches.",
        description: "Shown under the account search in the grant dialog when nothing matched.",
    },
    "perms.grant.level": {
        text: "Level",
        description: "Label over the rung picker in the grant dialog.",
    },
    "perms.cancel": {
        text: "Cancel",
        description: "Button that closes a dialog without saving.",
    },
    "perms.grant.submit": {
        text: "Grant {level}",
        description: "Confirm button of the grant dialog; {level} is the chosen rung, e.g. 'Grant Edit'.",
    },
    "perms.toast.grantCreated": {
        text: "Grant created",
        description: "Toast title after a new grant was made.",
    },
    "perms.toast.grantCreated.desc": {
        text: "{level} access on /{slug}.",
        description: "Toast body after a new grant was made; {level} is the rung and {slug} the tier list's URL name. Keep the slash.",
    },
    "perms.toast.grantFailed": {
        text: "Failed to grant",
        description: "Toast title after a new grant was rejected.",
    },
    "perms.close": {
        text: "Close",
        description: "Accessible name of the backdrop behind a dialog; clicking it closes the dialog.",
    },
    "perms.flair.kicker": {
        text: "Manage · flair",
        description: "Eyebrow at the top of the flair dialog. Keep the middle dot. Rendered uppercase.",
    },
    "perms.flair.title": {
        text: "Change flair on /{slug}",
        description: "Heading of the flair dialog; {slug} is the tier list's URL name. Keep the leading slash.",
    },
    "perms.flair.none": {
        text: "None",
        description: "Flair option that removes the tag from the list.",
    },
    "perms.save": {
        text: "Save",
        description: "Confirm button of the flair dialog.",
    },
    "perms.toast.flair": {
        text: "Flair updated",
        description: "Toast title after a list's flair was changed.",
    },
    "perms.toast.flair.desc": {
        text: "Flair updated on /{slug}.",
        description: "Toast body after a list's flair was changed; {slug} is the tier list's URL name. Keep the leading slash.",
    },
    "perms.toast.flairFailed": {
        text: "Failed to set flair",
        description: "Toast title after a flair change was rejected.",
    },
    "perms.delete.kicker": {
        text: "Manage · delete",
        description: "Eyebrow at the top of the delete dialog. Keep the middle dot. Rendered uppercase.",
    },
    "perms.delete.title": {
        text: "Delete /{slug}?",
        description: "Heading of the delete confirmation; {slug} is the tier list's URL name. Keep the leading slash and the question mark.",
    },
    "perms.delete.body": {
        text: "This is permanent. All tiers, placements, and version history for {title} are destroyed.",
        description: "Delete confirmation. {title} is the list's title, shown in bold. A 'placement' is one operator sitting in one tier.",
    },
    "perms.delete.submit": {
        text: "Delete tier list",
        description: "Destructive confirm button of the delete dialog.",
    },
    "perms.toast.deleted": {
        text: "Tier list deleted",
        description: "Toast title after a tier list was deleted.",
    },
    "perms.toast.deleted.desc": {
        text: "/{slug} is gone.",
        description: "Toast body after a tier list was deleted; {slug} is its former URL name. Keep the leading slash.",
    },
    "perms.toast.deleteFailed": {
        text: "Failed to delete",
        description: "Toast title after a deletion was rejected.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
