import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "stats";

export const messages = {
    "catalog.operators": {
        text: "Operators",
        description: "Catalog tile label: the game's playable characters. Rendered uppercase in a narrow tile.",
    },
    "catalog.operators.meta": {
        text: "playable + alters",
        description: "Sub-line under the operator count. An 'alter' is an alternate version of an existing operator; the game's own term.",
    },
    "catalog.skills": {
        text: "Skills",
        description: "Catalog tile label: operator combat skills. Rendered uppercase.",
    },
    "catalog.skills.meta": {
        text: "across all archetypes",
        description: "Sub-line under the skill count. An 'archetype' is an operator's sub-class in the game.",
    },
    "catalog.modules": {
        text: "Modules",
        description: "Catalog tile label: operator equipment upgrades. Rendered uppercase.",
    },
    "catalog.modules.meta": {
        text: "stage-3 unlocked",
        description: "Sub-line under the module count: counted at the third and final module upgrade stage.",
    },
    "catalog.skins": {
        text: "Skins",
        description: "Catalog tile label: alternate operator outfits. 'Skin' is the game's word for a cosmetic outfit. Rendered uppercase.",
    },
    "catalog.skins.meta": {
        text: "elite & seasonal",
        description: "Sub-line under the skin count, naming the two outfit lines the game sells.",
    },
    "catalog.stages": {
        text: "Stages",
        description: "Catalog tile label: playable game levels. Rendered uppercase.",
    },
    "catalog.stages.meta": {
        text: "across {count} zones",
        description: "Sub-line under the stage count. A zone groups stages. {count} is an already-formatted number and the label is always plural in the source.",
    },
    "catalog.enemies": {
        text: "Enemies",
        description: "Catalog tile label: enemy units. Rendered uppercase.",
    },
    "catalog.enemies.meta": {
        text: "indexed",
        description: "Sub-line under the enemy count: how many we hold data for.",
    },
    "catalog.tierLists": {
        text: "Tier lists",
        description: "Catalog tile label: community tier lists. Its value reads 'active/total'. Rendered uppercase.",
    },
    "catalog.tierLists.meta": {
        text: "{count} placements",
        description: "Sub-line under the tier-list tile. A placement is one operator placed on one tier. {count} is an already-formatted compact number and the label is always plural in the source.",
    },
    "catalog.rosters": {
        text: "Rosters",
        description: "Catalog tile label: player collections synced with the site. Rendered uppercase.",
    },
    "catalog.rosters.meta": {
        text: "Yostar-linked doctors",
        description: "Sub-line under the roster count. 'Yostar' is the game publisher's account system and 'Doctor' is the game's word for the player; both stay as-is.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
