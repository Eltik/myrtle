import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "admin";

export const messages = {
    "settings.kicker": {
        text: "Operate",
        description: "Eyebrow over the admin Settings page title, naming the admin section it belongs to. Rendered uppercase.",
    },
    "settings.title": {
        text: "Settings",
        description: "Title of the admin settings screen.",
    },
    "settings.sub": {
        text: "Service-wide configuration. Most values live in env vars and require a redeploy.",
        description: "Caption under the admin Settings title. 'Env vars' are the deployment's environment variables.",
    },
    "settings.probe.title": {
        text: "Runtime probe",
        description: "Heading of the card showing what the backend reports about itself right now.",
    },
    "settings.probe.desc": {
        text: "Live data from {endpoint}.",
        description: "Caption under the Runtime probe heading. {endpoint} is an endpoint path, shown in monospace.",
    },
    "settings.kv.cacheBackend": {
        text: "Cache backend",
        description: "Row label; its value is which cache implementation the backend uses.",
    },
    "settings.kv.cacheStatus": {
        text: "Cache status",
        description: "Row label; its value is whether the cache is reachable.",
    },
    "settings.kv.databaseStatus": {
        text: "Database status",
        description: "Row label; its value is whether the database is reachable.",
    },
    "settings.kv.serviceStatus": {
        text: "Service status",
        description: "Row label; its value is the backend's own overall verdict.",
    },
    "settings.kv.probeTimestamp": {
        text: "Probe timestamp",
        description: "Row label; its value is when the backend answered the health check.",
    },
    "settings.probe.failed": {
        text: "Probe failed.",
        description: "Shown in place of the runtime probe rows when the health check did not answer.",
    },
    "settings.user.title": {
        text: "Signed-in User",
        description: "Heading of the card describing the account currently signed in.",
    },
    "settings.user.desc": {
        text: "Your current session.",
        description: "Caption under the Signed-in User heading.",
    },
    "settings.kv.nickname": {
        text: "Nickname",
        description: "Row label for the signed-in account's display name.",
    },
    "settings.kv.uid": {
        text: "UID",
        description: "Row label for the in-game account number. 'UID' is the game's own abbreviation and stays as-is.",
    },
    "settings.kv.server": {
        text: "Server",
        description: "Row label for which Arknights game region the account belongs to.",
    },
    "settings.kv.role": {
        text: "Role",
        description: "Row label for the signed-in account's global admin role.",
    },
    "settings.kv.totalScore": {
        text: "Total score",
        description: "Row label for the account's computed collection score.",
    },
    "settings.badge.super": {
        text: "super",
        description: "Small badge beside the role row marking a super-admin.",
    },
    "settings.accountSettings": {
        text: "Account settings",
        description: "Button that leaves the admin panel for the signed-in account's own settings page.",
    },
    "settings.publicProfile": {
        text: "Public profile",
        description: "Button that opens the signed-in account's public profile in a new tab.",
    },
    "settings.gameData.title": {
        text: "Loaded game data",
        description: "Heading of the card counting what the backend has resident from the Arknights data files.",
    },
    "settings.gameData.desc": {
        text: "Snapshot of what the backend currently has resident from {dir}. Counts come from {endpoint} - they reflect the live in-memory dataset, so any drift here means the asset import is out of date and a redeploy or asset refresh is needed.",
        description: "Caption under the Loaded game data heading. {dir} is an environment-variable name and {endpoint} an endpoint path, both shown in monospace. Keep the hyphen.",
    },
    "settings.refresh": {
        text: "Refresh",
        description: "Button that re-fetches the game-data counts.",
    },
    "settings.gameData.error": {
        text: "Failed to load /admin/stats. Your account may not have tier_list_admin or above.",
        description: "Shown when the stats request was refused. The endpoint path and the role name are API identifiers and stay as-is.",
    },
    "settings.tile.operators": {
        text: "Operators",
        description: "Tile label counting loaded operators. 'Operator' is the game's word for a playable character. Rendered uppercase.",
    },
    "settings.tile.skills": {
        text: "Skills",
        description: "Tile label counting loaded operator skills. Rendered uppercase.",
    },
    "settings.tile.modules": {
        text: "Modules",
        description: "Tile label counting loaded modules. 'Module' is the game's own equipment system. Rendered uppercase.",
    },
    "settings.tile.skins": {
        text: "Skins",
        description: "Tile label counting loaded outfits. Rendered uppercase.",
    },
    "settings.tile.stages": {
        text: "Stages",
        description: "Tile label counting loaded stages. A stage is one playable level. Rendered uppercase.",
    },
    "settings.tile.zones": {
        text: "Zones",
        description: "Tile label counting loaded zones. A zone groups stages together. Rendered uppercase.",
    },
    "settings.tile.enemies": {
        text: "Enemies",
        description: "Tile label counting loaded enemy types. Rendered uppercase.",
    },
    "settings.snapshotComputed": {
        text: "Snapshot computed {when}",
        description: "Footer line under the game-data tiles. {when} is a relative time such as '2 minutes ago', shown in the foreground colour.",
    },
    "settings.cachedFor": {
        text: "(cached for ~60s)",
        description: "Aside after the snapshot time saying how long the backend caches these counts. '~60s' is about sixty seconds. Keep the parentheses.",
    },
    "settings.placementsAcross": {
        text: "{placements} placements across {lists} tier lists",
        description: "Footer total under the game-data tiles. A placement is one operator sitting in one tier. Both numbers are already formatted and may be a dash when unknown; the wording stays plural at any count.",
    },
    "settings.gameData.empty": {
        text: "No data.",
        description: "Shown in place of the game-data tiles when the stats request came back empty.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
