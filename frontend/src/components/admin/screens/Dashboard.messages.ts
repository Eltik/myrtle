import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "admin";

export const messages = {
    "dash.kicker": {
        text: "Overview",
        description: "Eyebrow over the admin Dashboard title, naming the admin section it belongs to. Rendered uppercase.",
    },
    "dash.title": {
        text: "Dashboard",
        description: "Title of the admin overview screen.",
    },
    "dash.sub": {
        text: "Service-level signals from the Rust backend - surfaced verbatim from {stats} and {health}.",
        description: "Sentence under the Dashboard title. {stats} and {health} are backend endpoint paths, shown in monospace. 'Rust' is the language the backend is written in. Keep the hyphen.",
    },
    "dash.refresh": {
        text: "Refresh",
        description: "Button that re-fetches the dashboard's stats and health probes.",
    },
    "dash.viewPublicStats": {
        text: "View public stats",
        description: "Button that opens the site's public statistics page in a new tab.",
    },
    "dash.statsError.title": {
        text: "Failed to load /admin/stats.",
        description: "Bold lead of the banner shown when the stats request was refused. The endpoint path stays as-is.",
    },
    "dash.statsError.body": {
        text: "Your account may not have {role} or above.",
        description: "Second sentence of the stats-error banner, after the bold lead: the account needs that role or a higher one. {role} is a role name, shown in monospace.",
    },
    "dash.tile.totalUsers": {
        text: "Total users",
        description: "Stat tile label counting every registered account. Rendered uppercase.",
    },
    "dash.tile.rostersSynced": {
        text: "Rosters synced",
        description: "Stat tile label counting players who have linked their account. A roster is a player's own operator collection. Rendered uppercase.",
    },
    "dash.tile.tierListsActive": {
        text: "Tier lists · active",
        description: "Stat tile label for how many tier lists are live. Keep the middle dot. Rendered uppercase.",
    },
    "dash.tile.of": {
        text: "of {count}",
        description: "Unit under the active tier-list count, naming the total it is part of, e.g. 'of 42'.",
    },
    "dash.tile.placements": {
        text: "Tier-list placements",
        description: "Stat tile label counting how many operators sit in a tier across all lists. Rendered uppercase.",
    },
    "dash.roles.title": {
        text: "Roles breakdown",
        description: "Heading of the card charting how many accounts hold each global role.",
    },
    "dash.roles.desc": {
        text: "From the {column} column - global access rungs.",
        description: "Caption under the Roles breakdown heading. {column} is a database column name, shown in monospace. A 'rung' is one step of the access ladder. Keep the hyphen.",
    },
    "dash.roles.manageUsers": {
        text: "Manage users",
        description: "Link from the Roles breakdown card to the user list. An arrow icon follows it.",
    },
    "dash.recent.title": {
        text: "Recently signed-up users",
        description: "Heading of the card listing the newest accounts.",
    },
    "dash.recent.desc": {
        text: "Last {count} players to authenticate. From {endpoint}.",
        description: "Caption under the Recently signed-up users heading. {endpoint} is a backend endpoint path, shown in monospace. 'Doctor' is what Arknights calls the player, and the wording stays plural at any count.",
    },
    "dash.th.doctor": {
        text: "Player",
        description: "Table column header for the account. 'Player' is the site's term; the game itself says 'Doctor'. Rendered uppercase.",
    },
    "dash.th.server": {
        text: "Server",
        description: "Table column header for which Arknights game region the account belongs to. Rendered uppercase.",
    },
    "dash.th.level": {
        text: "Level",
        description: "Table column header for the player's in-game level. Rendered uppercase.",
    },
    "dash.th.joined": {
        text: "Joined",
        description: "Table column header for when the account first signed in. Rendered uppercase.",
    },
    "dash.recent.empty": {
        text: "No recent signups.",
        description: "Empty state in the Recently signed-up users card.",
    },
    "dash.user.uid": {
        text: "UID {uid}",
        description: "An account's in-game number under its nickname. 'UID' is the game's own abbreviation and stays as-is.",
    },
    "dash.health.title": {
        text: "Service health",
        description: "Heading of the card summarising the backend health probe.",
    },
    "dash.health.healthy": {
        text: "healthy",
        description: "Badge on the Service health card when the backend reports itself in good order.",
    },
    "dash.health.degraded": {
        text: "degraded",
        description: "Badge on the Service health card when the backend is answering but reports a problem.",
    },
    "dash.health.postgres": {
        text: "Postgres",
        description: "Row label for the database probe. 'Postgres' is the database product and stays as-is.",
    },
    "dash.health.cache": {
        text: "Cache ·",
        description: "Row label for the cache probe; the cache implementation's name follows after a space. Keep the middle dot.",
    },
    "dash.health.gameData": {
        text: "Game data",
        description: "Row label for how much Arknights data the backend has loaded.",
    },
    "dash.health.ops": {
        text: "{count} ops",
        description: "How many operators are loaded, e.g. '431 ops'. 'ops' abbreviates operators, the game's playable characters. The number is already formatted.",
    },
    "dash.health.ms": {
        text: "{ms} ms",
        description: "A response time in milliseconds, e.g. '12.4 ms'. The number is already formatted.",
    },
    "dash.health.roundTrip": {
        text: "Round-trip",
        description: "Row label for how long the whole health request took.",
    },
    "dash.health.unavailable": {
        text: "Health probe unavailable.",
        description: "Shown in place of the Service health rows when the probe did not answer.",
    },
    "dash.signedIn.title": {
        text: "Signed in as",
        description: "Heading of the card describing the account currently signed in.",
    },
    "dash.signedIn.profile": {
        text: "Profile",
        description: "Button that opens the signed-in account's public profile in a new tab. An external-link icon follows it.",
    },
    "dash.quick.title": {
        text: "Quick links",
        description: "Heading of the card listing shortcuts to the other admin screens.",
    },
    "dash.quick.when.manage": {
        text: "manage",
        description: "Small label marking a quick link as one of the content-editing screens. Rendered uppercase.",
    },
    "dash.quick.when.operate": {
        text: "operate",
        description: "Small label marking a quick link as one of the service-inspection screens. Rendered uppercase.",
    },
    "dash.quick.permissions": {
        text: "Tier list permissions",
        description: "Quick link to the screen that manages who may edit each tier list.",
    },
    "dash.quick.permissions.who": {
        text: "View / Edit / Publish / Admin",
        description: "Caption under the tier-list permissions quick link, naming the four rungs of the permission ladder. Keep the slashes.",
    },
    "dash.quick.official": {
        text: "Official tier lists",
        description: "Quick link to the editor for site-run tier lists.",
    },
    "dash.quick.official.who": {
        text: "Flair-tagged lists in the Official rail",
        description: "Caption under the official tier lists quick link. A 'flair' is the coloured tag on a list; the 'Official rail' is the row of them on the public site.",
    },
    "dash.quick.notes": {
        text: "Operator notes",
        description: "Quick link to the per-operator guidance editor.",
    },
    "dash.quick.notes.who": {
        text: "Community guidance per operator",
        description: "Caption under the operator notes quick link. 'Operator' is the game's word for a playable character.",
    },
    "dash.quick.health": {
        text: "Health & cache",
        description: "Quick link to the backend health probe screen. Keep the ampersand.",
    },
    "dash.quick.health.who": {
        text: "Redis + Postgres probes",
        description: "Caption under the health quick link. 'Redis' and 'Postgres' are product names and stay as-is.",
    },
    "dash.quick.audit": {
        text: "Audit log",
        description: "Quick link to the append-only trail of admin edits.",
    },
    "dash.quick.audit.who": {
        text: "Permission grants + note edits",
        description: "Caption under the audit log quick link, naming what the log records.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
