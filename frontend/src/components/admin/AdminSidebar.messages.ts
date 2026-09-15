import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "admin";

export const messages = {
    "sidebar.nav.dashboard": {
        text: "Dashboard",
        description: "Admin sidebar link to the overview screen.",
    },
    "sidebar.nav.users": {
        text: "Users",
        description: "Admin sidebar link to the user list. The count beside it is how many accounts exist.",
    },
    "sidebar.nav.officialTierLists": {
        text: "Official tier lists",
        description: "Admin sidebar link to the editor for site-run tier lists.",
    },
    "sidebar.nav.tierLists": {
        text: "Tier Lists",
        description: "Admin sidebar link to the screen that manages every tier list and its permission grants.",
    },
    "sidebar.nav.operatorNotes": {
        text: "Operator notes",
        description: "Admin sidebar link to the per-operator guidance editor. 'Operator' is the game's word for a playable character.",
    },
    "sidebar.nav.translations": {
        text: "Translations",
        description: "Admin sidebar link to the translation workspace. The count beside it is the untranslated-plus-stale backlog.",
    },
    "sidebar.nav.health": {
        text: "Health & cache",
        description: "Admin sidebar link to the backend health probe screen. Keep the ampersand.",
    },
    "sidebar.nav.audit": {
        text: "Audit log",
        description: "Admin sidebar link to the append-only trail of admin edits.",
    },
    "sidebar.nav.settings": {
        text: "Settings",
        description: "Admin sidebar link to the service-wide settings screen.",
    },
    "sidebar.closeBackdrop": {
        text: "Close sidebar",
        description: "Accessible name of the backdrop behind the mobile admin sidebar; tapping it closes the sidebar.",
    },
    "sidebar.home": {
        text: "myrtle.moe admin home",
        description: "Accessible name of the logo link at the top of the admin sidebar. 'myrtle.moe' is the site name and stays as-is.",
    },
    "sidebar.badge": {
        text: "admin",
        description: "Small label under the site name marking this as the admin panel. Rendered uppercase.",
    },
    "sidebar.closeMenu": {
        text: "Close menu",
        description: "Accessible name of the X button that closes the admin sidebar on a narrow screen.",
    },
    "sidebar.group.manage": {
        text: "Manage",
        description: "Heading over the sidebar links that edit content. Rendered uppercase.",
    },
    "sidebar.group.operate": {
        text: "Operate",
        description: "Heading over the sidebar links that inspect the running service. Rendered uppercase.",
    },
    "sidebar.profile": {
        text: "View your profile",
        description: "Accessible name of the signed-in account card at the bottom of the admin sidebar.",
    },
    "sidebar.guest": {
        text: "Guest",
        description: "Stand-in nickname on the account card when nobody is signed in.",
    },
    "sidebar.backToSite": {
        text: "← back to site",
        description: "Link out of the admin panel to the public site. Keep the leading arrow.",
    },
} satisfies MessageMap;

// `dynamic`: the nav labels are stored as `labelKey` on the nav item arrays and
// resolved by the row component as `t(item.labelKey)`, so the extractor has no
// literal call site to match them against.
export const { keys } = defineMessages({ namespace, messages, dynamic: true });
