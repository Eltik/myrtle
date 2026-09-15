import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "admin";

export const messages = {
    "health.kicker": {
        text: "Operate",
        description: "Eyebrow over the Health & cache page title, naming the admin section it belongs to. Rendered uppercase.",
    },
    "health.title": {
        text: "Health & cache",
        description: "Title of the admin screen that probes the backend. Keep the ampersand.",
    },
    "health.sub": {
        text: "Live probe of the Rust backend - {health} and the public {stats} snapshot.",
        description: "Sentence under the Health & cache title. {health} and {stats} are endpoint paths, shown in monospace. 'Rust' is the language the backend is written in. Keep the hyphen.",
    },
    "health.reprobe": {
        text: "Re-probe",
        description: "Button that re-runs the backend health check.",
    },
    "health.tile.postgres": {
        text: "Postgres",
        description: "Label of the tile showing the database round-trip time. 'Postgres' is the database product and stays as-is. Rendered uppercase.",
    },
    "health.tile.cache": {
        text: "Cache · {backend}",
        description: "Label of the cache tile; {backend} is the cache implementation the backend reports, e.g. 'redis'. Rendered uppercase.",
    },
    "health.tile.roundTrip": {
        text: "Round-trip",
        description: "Label of the tile showing how long the whole health request took. Rendered uppercase.",
    },
    "health.ms": {
        text: "ms",
        description: "Unit after a response time: milliseconds.",
    },
    "health.status.unknown": {
        text: "unknown",
        description: "Stand-in connection status when the probe returned nothing.",
    },
    "health.badge.healthy": {
        text: "healthy",
        description: "Badge shown when the service reports itself in good order.",
    },
    "health.badge.degraded": {
        text: "degraded",
        description: "Badge shown when the service is answering but reports a problem.",
    },
    "health.gameData.title": {
        text: "Game data",
        description: "Heading of the card counting what the backend has loaded from the Arknights data files.",
    },
    "health.gameData.desc": {
        text: "Backed by the in-memory {type}. Counts from {endpoint}.",
        description: "Caption under the Game data heading. {type} is a Rust type name and {endpoint} an endpoint path, both shown in monospace.",
    },
    "health.kv.operators": {
        text: "Operators",
        description: "Row label counting loaded operators. 'Operator' is the game's word for a playable character.",
    },
    "health.kv.skills": {
        text: "Skills",
        description: "Row label counting loaded operator skills.",
    },
    "health.kv.modules": {
        text: "Modules",
        description: "Row label counting loaded modules. 'Module' is the game's own equipment system.",
    },
    "health.kv.skins": {
        text: "Skins",
        description: "Row label counting loaded outfits.",
    },
    "health.kv.stages": {
        text: "Stages",
        description: "Row label counting loaded stages. A stage is one playable level.",
    },
    "health.kv.zones": {
        text: "Zones",
        description: "Row label counting loaded zones. A zone groups stages together.",
    },
    "health.kv.enemies": {
        text: "Enemies",
        description: "Row label counting loaded enemy types.",
    },
    "health.statsUnavailable": {
        text: "/stats endpoint unavailable.",
        description: "Shown in place of the counts when the public stats endpoint did not answer. The path stays as-is.",
    },
    "health.snapshot.title": {
        text: "Stats snapshot",
        description: "Heading of the card showing platform totals.",
    },
    "health.snapshot.desc": {
        text: "Computed at {at}.",
        description: "Caption under the Stats snapshot heading; {at} is an already-formatted date and time, or a dash when unknown.",
    },
    "health.kv.tierListsTotal": {
        text: "Tier lists · total",
        description: "Row label counting every tier list. Keep the middle dot.",
    },
    "health.kv.tierListsActive": {
        text: "Tier lists · active",
        description: "Row label counting tier lists that are not archived. Keep the middle dot.",
    },
    "health.kv.tierListVersions": {
        text: "Tier list versions",
        description: "Row label counting published snapshots of tier lists.",
    },
    "health.kv.tierListPlacements": {
        text: "Tier list placements",
        description: "Row label counting how many operators sit in a tier across all lists.",
    },
    "health.kv.rostersSynced": {
        text: "Rosters synced",
        description: "Row label counting players who have linked their account. A roster is a player's own operator collection.",
    },
    "health.probe.title": {
        text: "Probe timestamp",
        description: "Heading of the card showing when the health probe last ran.",
    },
    "health.probe.desc": {
        text: "Auto-refreshes every 30s.",
        description: "Caption under the Probe timestamp heading. '30s' is thirty seconds.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
