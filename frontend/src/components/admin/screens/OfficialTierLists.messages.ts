import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "admin";

export const messages = {
    "official.kicker": {
        text: "Manage",
        description: "Eyebrow over the Official tier lists page title, naming the admin section it belongs to. Rendered uppercase.",
    },
    "official.title": {
        text: "Official tier lists",
        description: "Title of the admin screen that manages the site's own tier lists.",
    },
    "official.sub": {
        text: "Tier lists with {filter} surface in the Official rail. Same editor as community; {role} writes directly without per-list grants.",
        description: "Sentence under the Official tier lists title. {filter} is a column-and-value pair and {role} a role name, both shown in monospace. The 'Official rail' is the row of site-run lists on the public site; 'community' lists are the player-made ones. That role needs no individual permission grant.",
    },
    "official.viewPublicRail": {
        text: "View public rail",
        description: "Button that opens the public tier-list page in a new tab.",
    },
    "official.newList": {
        text: "New official list",
        description: "Button that opens the dialog for creating a site-run tier list.",
    },
    "official.searchPlaceholder": {
        text: "Search by title or slug…",
        description: "Prompt in the official tier-list search box. A 'slug' is the list's URL name. Keep the ellipsis character.",
    },
    "official.filter.all": {
        text: "All",
        description: "Filter tab that shows every official tier list.",
    },
    "official.filter.trending": {
        text: "Trending",
        description: "Filter tab that shows only the official tier lists currently getting traffic.",
    },
    "official.filter.draft": {
        text: "Draft",
        description: "Filter tab that shows only official tier lists that are not live yet.",
    },
    "official.empty": {
        text: "No official tier lists yet - create one.",
        description: "Empty state in the official tier-list table. Keep the hyphen.",
    },
    "official.th.title": {
        text: "Title",
        description: "Table column header for a tier list's name. Rendered uppercase.",
    },
    "official.th.flair": {
        text: "Flair",
        description: "Table column header for the coloured tag on a list. Rendered uppercase.",
    },
    "official.th.trending": {
        text: "Trending",
        description: "Table column header for whether a list is currently getting traffic. Rendered uppercase.",
    },
    "official.th.tiers": {
        text: "Tiers",
        description: "Table column header counting a list's tiers. Rendered uppercase.",
    },
    "official.th.placements": {
        text: "Placements",
        description: "Table column header counting how many operators sit in a tier on this list. Rendered uppercase.",
    },
    "official.th.updated": {
        text: "Updated",
        description: "Table column header for when the list last changed. Rendered uppercase.",
    },
    "official.th.views": {
        text: "Views (24h)",
        description: "Table column header counting page views in the last day. '24h' is twenty-four hours. Rendered uppercase.",
    },
    "official.badge.trending": {
        text: "trending",
        description: "Badge on a list that is currently getting traffic.",
    },
    "official.rowActions": {
        text: "Row actions",
        description: "Accessible name of the three-dots button that opens a tier list's action menu.",
    },
    "official.menu.viewPublic": {
        text: "View public page",
        description: "Menu item that opens the tier list's public page in a new tab.",
    },
    "official.menu.openEditor": {
        text: "Open editor",
        description: "Menu item that opens the tier list in the editor.",
    },
    "official.menu.changeFlair": {
        text: "Change flair",
        description: "Menu item that opens the flair picker. A 'flair' is the coloured tag shown on a list.",
    },
    "official.menu.publish": {
        text: "Publish version",
        description: "Menu item that opens the dialog for snapshotting the list as a new public version.",
    },
    "official.menu.delete": {
        text: "Delete tier list",
        description: "Destructive menu item that opens the delete confirmation.",
    },
    "official.close": {
        text: "Close",
        description: "Accessible name of the backdrop behind a dialog; clicking it closes the dialog.",
    },
    "official.cancel": {
        text: "Cancel",
        description: "Button that closes a dialog without saving.",
    },
    "official.flair.kicker": {
        text: "Manage · flair",
        description: "Eyebrow at the top of the flair dialog. Keep the middle dot. Rendered uppercase.",
    },
    "official.flair.title": {
        text: "Change flair on /{slug}",
        description: "Heading of the flair dialog; {slug} is the tier list's URL name. Keep the leading slash.",
    },
    "official.flair.none": {
        text: "None",
        description: "Flair option that removes the tag from the list.",
    },
    "official.save": {
        text: "Save",
        description: "Confirm button of the flair dialog.",
    },
    "official.toast.flair": {
        text: "Flair updated",
        description: "Toast title after a list's flair was changed.",
    },
    "official.toast.flair.desc": {
        text: "Flair updated on /{slug}.",
        description: "Toast body after a list's flair was changed; {slug} is the tier list's URL name. Keep the leading slash.",
    },
    "official.toast.flairFailed": {
        text: "Failed to set flair",
        description: "Toast title after a flair change was rejected.",
    },
    "official.delete.kicker": {
        text: "Manage · delete",
        description: "Eyebrow at the top of the delete dialog. Keep the middle dot. Rendered uppercase.",
    },
    "official.delete.title": {
        text: "Delete /{slug}?",
        description: "Heading of the delete confirmation; {slug} is the tier list's URL name. Keep the leading slash and the question mark.",
    },
    "official.delete.body": {
        text: "This is permanent. All tiers, placements, and version history for {title} are destroyed.",
        description: "Delete confirmation. {title} is the list's title, shown in bold. A 'placement' is one operator sitting in one tier.",
    },
    "official.delete.submit": {
        text: "Delete tier list",
        description: "Destructive confirm button of the delete dialog.",
    },
    "official.toast.deleted": {
        text: "Tier list deleted",
        description: "Toast title after a tier list was deleted.",
    },
    "official.toast.deleted.desc": {
        text: "/{slug} is gone.",
        description: "Toast body after a tier list was deleted; {slug} is its former URL name. Keep the leading slash.",
    },
    "official.toast.deleteFailed": {
        text: "Failed to delete",
        description: "Toast title after a deletion was rejected.",
    },
    "official.publish.kicker": {
        text: "Manage · publish",
        description: "Eyebrow at the top of the publish dialog. Keep the middle dot. Rendered uppercase.",
    },
    "official.publish.title": {
        text: "Publish a new version of /{slug}",
        description: "Heading of the publish dialog; {slug} is the tier list's URL name. Keep the leading slash.",
    },
    "official.publish.desc": {
        text: "Snapshots the current tier layout. Visible in version history.",
        description: "Caption in the publish dialog explaining what publishing does.",
    },
    "official.publish.changelog": {
        text: "Changelog (optional)",
        description: "Label over the free-text field describing what changed in this version. Keep the parentheses.",
    },
    "official.publish.changelogPlaceholder": {
        text: "What changed?",
        description: "Prompt inside the empty changelog field.",
    },
    "official.publish.submit": {
        text: "Publish",
        description: "Confirm button of the publish dialog. A verb.",
    },
    "official.toast.published": {
        text: "Version published",
        description: "Toast title after a new version went live.",
    },
    "official.toast.published.desc": {
        text: "New version of /{slug} is live.",
        description: "Toast body after a new version went live; {slug} is the tier list's URL name. Keep the leading slash.",
    },
    "official.toast.publishFailed": {
        text: "Failed to publish",
        description: "Toast title after publishing was rejected.",
    },
    "official.new.kicker": {
        text: "Manage · official tier lists",
        description: "Eyebrow at the top of the new-list dialog. Keep the middle dot. Rendered uppercase.",
    },
    "official.new.title": {
        text: "New official tier list",
        description: "Heading of the dialog for creating a site-run tier list.",
    },
    "official.new.desc": {
        text: "Creates a draft via {endpoint} with {field}. Add tiers + flair in the editor.",
        description: "Caption in the new-list dialog: the rest of the setup happens in the tier-list editor. {endpoint} is an endpoint and {field} a column-and-value pair, both shown in monospace.",
    },
    "official.new.titleLabel": {
        text: "Title",
        description: "Label over the new list's name field.",
    },
    "official.new.titlePlaceholder": {
        text: "e.g. Endgame DPS rankings",
        description: "Example name inside the empty title field. 'DPS' is damage per second, the community's own abbreviation.",
    },
    "official.new.titleHint": {
        text: "Shown on browse cards and the public detail page. The slug is auto-generated.",
        description: "Helper text under the title field. A 'slug' is the list's URL name.",
    },
    "official.new.descLabel": {
        text: "Description (optional)",
        description: "Label over the new list's blurb field. Keep the parentheses.",
    },
    "official.new.descPlaceholder": {
        text: "A short blurb for the public page.",
        description: "Prompt inside the empty description field.",
    },
    "official.new.flairs": {
        text: "Available flairs",
        description: "Label over the read-only list of flairs that could be applied later. A 'flair' is the coloured tag shown on a list.",
    },
    "official.new.flairsHint": {
        text: "Set the flair from the tier list editor after creating the draft.",
        description: "Helper text under the available flairs.",
    },
    "official.new.submit": {
        text: "Create draft",
        description: "Confirm button of the new-list dialog.",
    },
    "official.toast.created": {
        text: "Draft created",
        description: "Toast title after a new official tier list was created.",
    },
    "official.toast.created.desc": {
        text: "New official tier list /{slug} created. Open the editor to add tiers.",
        description: "Toast body after a new official tier list was created; {slug} is its URL name. Keep the leading slash.",
    },
    "official.toast.createFailed": {
        text: "Failed to create",
        description: "Toast title after creating a list was rejected.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
