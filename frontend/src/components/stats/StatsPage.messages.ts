import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "stats";

export const messages = {
    "page.kpi.operators": {
        text: "Operators indexed",
        description: "Headline tile label: how many operators (playable characters) the site has data for. Rendered uppercase in a narrow tile.",
    },
    "page.kpi.operators.meta": {
        text: "{skills} skills · {modules} modules",
        description: "Sub-line under the operator count. Both values are already-formatted numbers; 'modules' are operator equipment upgrades.",
    },
    "page.kpi.tierLists": {
        text: "Active tier lists",
        description: "Headline tile label: how many community tier lists are currently published. Rendered uppercase.",
    },
    "page.kpi.tierLists.meta": {
        text: "{total} total · {versions} versions",
        description: "Sub-line under the active-tier-list count. A version is a saved revision of a list. Both values are already-formatted numbers.",
    },
    "page.kpi.rosters": {
        text: "Rosters synced",
        description: "Headline tile label: how many players have linked their game account. Rendered uppercase.",
    },
    "page.kpi.rosters.meta": {
        text: "Yostar-linked doctors",
        description: "Sub-line under the roster count. 'Yostar' is the game publisher's account system and 'Doctor' is the game's word for the player; both stay as-is.",
    },
    "page.error.load": {
        text: "Couldn’t load site stats.",
        description: "Bold lead of the stats-page error banner; the request's own message follows it.",
    },
    "page.error.unknown": {
        text: "Unknown error.",
        description: "Stand-in used in the stats-page error banner when the failure carried no message.",
    },
    "page.catalog.kicker": {
        text: "Game catalog",
        description: "Eyebrow label above the game-catalog section of the stats page. Rendered uppercase.",
    },
    "page.catalog.title": {
        text: "What we know about Terra.",
        description: "Heading of the game-catalog section. 'Terra' is the name of the game's world and is not translated.",
    },
    "page.catalog.blurb": {
        text: "A faithful mirror of Hypergryph’s gamedata, recomputed on every build.",
        description: "Caption under the game-catalog heading. 'Hypergryph' is the game's developer and stays as-is.",
    },
    "page.community.kicker": {
        text: "Tier lists · community",
        description: "Eyebrow label above the community tier-list section of the stats page. Rendered uppercase; the middle dot is a separator.",
    },
    "page.community.title": {
        text: "The community’s working notes.",
        description: "Heading of the community tier-list section of the stats page.",
    },
    "page.community.blurb": {
        text: "Counted across every public list. {versions} are saved revisions; {placements} are individual operator-on-tier rows.",
        description: "Caption under the community tier-list heading. {versions} and {placements} are the two bolded terms this sentence defines; move them wherever the sentence needs them.",
    },
    "page.community.blurb.versions": {
        text: "Versions",
        description: "Bolded term substituted into page.community.blurb as {versions}, defined there by the clause 'are saved revisions'.",
    },
    "page.community.blurb.placements": {
        text: "placements",
        description: "Bolded term substituted into page.community.blurb as {placements}: one row is one operator placed on one tier.",
    },
    "page.cell.totalLists": {
        text: "Total lists",
        description: "Tile label: every community tier list ever published. Rendered uppercase.",
    },
    "page.cell.totalLists.meta": {
        text: "{count} archived",
        description: "Sub-line under the total-list count. {count} is an already-formatted number and the label is always plural in the source.",
    },
    "page.cell.active": {
        text: "Active",
        description: "Tile label: community tier lists that are currently published, as opposed to archived. Rendered uppercase.",
    },
    "page.cell.active.meta": {
        text: "{percent}% of total",
        description: "Sub-line under the active-list count. {percent} is an already-rounded whole number, without its percent sign.",
    },
    "page.cell.versions": {
        text: "Versions saved",
        description: "Tile label: saved revisions across every community tier list. Rendered uppercase.",
    },
    "page.cell.placements": {
        text: "Placements",
        description: "Tile label: individual operator-on-tier rows across every community tier list. Rendered uppercase.",
    },
    "page.cell.avgPerList": {
        text: "avg {avg} per list",
        description: "Sub-line under a tier-list tile, e.g. 'avg 2.4 per list'. 'avg' is short for average; {avg} is an already-formatted number.",
    },
    "page.footer": {
        text: "Looking for gacha rates, most-pulled operators, or pull timing? Those live on {link}. This page is the meta-count - just the shape of what we host.",
        description: "Closing note on the stats page. {link} is a link whose label is the literal URL path /gacha/community; move it wherever the sentence needs it. 'Gacha' is the game's randomised recruitment system and a 'pull' is one draw from it.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
