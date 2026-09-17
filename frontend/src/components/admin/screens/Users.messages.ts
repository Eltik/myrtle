import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "admin";

export const messages = {
    "users.kicker": {
        text: "Manage",
        description: "Eyebrow over the admin Users page title, naming the admin section it belongs to. Rendered uppercase.",
    },
    "users.title": {
        text: "Users",
        description: "Title of the admin screen listing every account.",
    },
    "users.sub": {
        text: "Players with synced profiles. Pulled from {search}. Roles write to the {table} table via {endpoint} and only take effect for the target once their token refreshes.",
        description: "Sentence under the Users title: the person whose role changed keeps their old access until their sign-in token is renewed. {search} and {endpoint} are endpoint paths and {table} a database table name, all shown in monospace. 'Player' is the site's term; the game itself says 'Doctor'.",
    },
    "users.refresh": {
        text: "Refresh",
        description: "Button that re-fetches the user list.",
    },
    "users.searchPlaceholder": {
        text: "Search by nickname…",
        description: "Prompt in the user search box. Keep the ellipsis character.",
    },
    "users.allServers": {
        text: "All servers",
        description: "Filter tab that drops the game-region filter. The other tabs are the region codes themselves.",
    },
    "users.countShown": {
        text: "{filtered} of {total} shown",
        description: "How many accounts survive the current filter.",
    },
    "users.emptyServer": {
        text: "No public profiles on the {server} server yet.",
        description: "Empty state when a game region has no public profiles; {server} is a region code such as EN or JP.",
    },
    "users.emptyAll": {
        text: "No users match.",
        description: "Empty state when the search matched no accounts.",
    },
    "users.th.doctor": {
        text: "Player",
        description: "User table column header for the account. 'Player' is the site's term; the game itself says 'Doctor'. Rendered uppercase.",
    },
    "users.th.server": {
        text: "Server",
        description: "User table column header for the account's game region. Rendered uppercase.",
    },
    "users.th.level": {
        text: "Level",
        description: "User table column header for the player's in-game level. Rendered uppercase.",
    },
    "users.th.score": {
        text: "Score",
        description: "User table column header for the computed collection score. Rendered uppercase.",
    },
    "users.th.grade": {
        text: "Grade",
        description: "User table column header for the letter grade derived from the score. Rendered uppercase.",
    },
    "users.th.role": {
        text: "Role",
        description: "User table column header for the account's global admin role. Rendered uppercase.",
    },
    "users.uid": {
        text: "UID",
        description: "Label before an in-game account number. 'UID' is the game's own abbreviation and stays as-is.",
    },
    "users.open": {
        text: "Open",
        description: "Button on a user row that opens that public profile in a new tab. A verb.",
    },
    "users.closeDrawer": {
        text: "Close drawer",
        description: "Accessible name of the backdrop behind the user detail drawer; clicking it closes the drawer.",
    },
    "users.close": {
        text: "Close",
        description: "Accessible name of the X button in the user detail drawer.",
    },
    "users.level": {
        text: " · Level {level}",
        description: "Appended to the account line in the detail drawer when the player's level is known. Keep the leading space and middle dot.",
    },
    "users.gradeBadge": {
        text: "grade {grade}",
        description: "Badge in the user drawer showing the letter grade derived from the collection score, e.g. 'grade A'.",
    },
    "users.public": {
        text: "public",
        description: "Badge marking a profile as visible to everyone.",
    },
    "users.private": {
        text: "private",
        description: "Badge marking a profile as hidden from everyone else.",
    },
    "users.profile.title": {
        text: "Profile",
        description: "Heading of the card listing the account's collection figures.",
    },
    "users.score.total": {
        text: "Total score",
        description: "Row label for the account's computed collection score.",
    },
    "users.score.operators": {
        text: "Operators",
        description: "Row label counting the operators the account owns. 'Operator' is the game's word for a playable character.",
    },
    "users.score.items": {
        text: "Items",
        description: "Row label counting the items in the account's inventory.",
    },
    "users.score.skins": {
        text: "Skins",
        description: "Row label counting the outfits the account owns.",
    },
    "users.actions.title": {
        text: "Actions",
        description: "Heading of the card holding the buttons that act on this account.",
    },
    "users.openPublicProfile": {
        text: "Open public profile",
        description: "Button in the user drawer that opens that account's public page in a new tab.",
    },
    "users.globalRole": {
        text: "Global role",
        description: "Label over the control that changes the account's site-wide role.",
    },
    "users.globalRole.hint": {
        text: "Per-locale translation grants live on the Translations screen and take effect immediately; this role only opens the door.",
        description: "Helper text under the global role control: the role allows translating, but which languages is decided elsewhere.",
    },
    "users.resync": {
        text: "Force-resync and session-revoke still require direct DB / {endpoint} access - not yet exposed as REST endpoints.",
        description: "Note at the bottom of the user drawer. {endpoint} is an endpoint path, shown in monospace. 'DB' abbreviates database. Keep the hyphen.",
    },
    "users.roleError.self": {
        text: "You can't change your own role - another super-admin or the manage_permissions CLI has to do it.",
        description: "Error shown when a super-admin tries to change their own role. 'manage_permissions' is a command name and stays as-is.",
    },
    "users.toast.roleUpdated": {
        text: "Role updated",
        description: "Toast title after a role change succeeded.",
    },
    "users.toast.roleUpdated.desc": {
        text: "{name} is now {role}.",
        description: "Toast body after a role change; {name} is the account's nickname or number and {role} is the API role identifier, which stays as-is.",
    },
    "users.toast.roleFailed": {
        text: "Failed to change role",
        description: "Toast title after a role change was rejected.",
    },
    "users.you": {
        text: "you",
        description: "Marker beside the signed-in admin's own row, whose role cannot be changed from here.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
