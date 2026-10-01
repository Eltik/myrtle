import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tierLists";

export const messages = {
    "entity.kinds.operator": {
        text: "Operators",
        description: "Name of a kind of thing a tier list can rank: playable characters. Plural. Used on the editor's pool tabs and in the list of kinds a list offers.",
    },
    "entity.kinds.class": {
        text: "Classes",
        description: "Name of a kind of thing a tier list can rank: the eight operator classes (Vanguard, Guard, and so on). Plural. Used on pool tabs and in the kinds settings.",
    },
    "entity.kinds.subclass": {
        text: "Subclasses",
        description: "Name of a kind of thing a tier list can rank: operator subclasses (archetypes such as Charger or Lord). Plural. Used on pool tabs and in the kinds settings.",
    },
    "entity.kinds.faction": {
        text: "Factions",
        description: "Name of a kind of thing a tier list can rank: nations, groups and teams operators belong to. Plural. Used on pool tabs and in the kinds settings.",
    },
    "entity.kinds.enemy": {
        text: "Enemies",
        description: "Name of a kind of thing a tier list can rank: enemies from the game's enemy handbook. Plural. Used on pool tabs and in the kinds settings.",
    },
    "entity.kinds.event": {
        text: "Events",
        description: "Name of a kind of thing a tier list can rank: in-game events (side stories, vignettes and the like). Plural. Used on pool tabs and in the kinds settings.",
    },
    "entity.kinds.stronghold_bond": {
        text: "Stronghold bonds",
        description: "Name of a kind of thing a tier list can rank: the bonds of Stronghold Protocol, the game's auto-chess mode. Plural. Used on pool tabs and in the kinds settings.",
    },
    "entity.kind.operator": {
        text: "Operator",
        description: "Singular kind label shown small above an entity's name in its hover card.",
    },
    "entity.kind.class": {
        text: "Class",
        description: "Singular kind label shown small above a class's name in its hover card.",
    },
    "entity.kind.subclass": {
        text: "Subclass",
        description: "Singular kind label shown small above a subclass's name in its hover card.",
    },
    "entity.kind.faction": {
        text: "Faction",
        description: "Singular kind label shown small above a faction's name in its hover card.",
    },
    "entity.kind.enemy": {
        text: "Enemy",
        description: "Singular kind label shown small above an enemy's name in its hover card.",
    },
    "entity.kind.event": {
        text: "Event",
        description: "Singular kind label shown small above an event's name in its hover card.",
    },
    "entity.kind.stronghold_bond": {
        text: "Stronghold bond",
        description: "Singular kind label shown small above a Stronghold Protocol bond's name in its hover card.",
    },
    "entity.enemyLevel.normal": {
        text: "Normal",
        description: "Enemy rank in the game's enemy handbook: an ordinary enemy. Used as a pool filter and in hover cards.",
    },
    "entity.enemyLevel.elite": {
        text: "Elite",
        description: "Enemy rank in the game's enemy handbook: an elite enemy. Used as a pool filter and in hover cards.",
    },
    "entity.enemyLevel.boss": {
        text: "Leader",
        description: "Enemy rank in the game's enemy handbook: a boss. The English game calls this rank 'Leader'. Used as a pool filter and in hover cards.",
    },
    "entity.factionLevel.nation": {
        text: "Nation",
        description: "Faction level: a nation or state, such as Victoria. Used as a pool filter and in hover cards.",
    },
    "entity.factionLevel.group": {
        text: "Group",
        description: "Faction level: an organisation inside a nation, such as Rhine Lab. Used as a pool filter and in hover cards.",
    },
    "entity.factionLevel.team": {
        text: "Team",
        description: "Faction level: a small team, such as Team Rainbow. Used as a pool filter and in hover cards.",
    },
    "entity.bondType.season": {
        text: "Faction bond",
        description: "Stronghold Protocol bond type: a bond that gathers operators of some factions. Used as a pool filter and in hover cards.",
    },
    "entity.bondType.regular": {
        text: "Trait bond",
        description: "Stronghold Protocol bond type: a bond built around a shared trait rather than a faction. Used as a pool filter and in hover cards.",
    },
    "entity.eventType.SIDESTORY": {
        text: "Side Story",
        description: "Event category as the game's Archives files it: a full side story event. Used as a pool filter and in hover cards.",
    },
    "entity.eventType.MINISTORY": {
        text: "Vignette",
        description: "Event category as the game's Archives files it: a short story collection event. Used as a pool filter and in hover cards.",
    },
    "entity.eventType.BRANCHLINE": {
        text: "Intermezzo",
        description: "Event category as the game's Archives files it: a branch story between main chapters. Used as a pool filter and in hover cards.",
    },
    "entity.eventType.NONE": {
        text: "Other",
        description: "Event category for events the game's Archives does not file on a story shelf (modes, contracts, collaborations). Used as a pool filter and in hover cards.",
    },
    "entity.event.rerun": {
        text: "Rerun",
        description: "Marks an event that is a rerun or retrospective of an earlier one. Used as a pool filter and in hover cards.",
    },
    "entity.tile.label": {
        text: "{name}, {kind}",
        description: "Accessible name of a non-operator tile: the entity's name from the game data, then its kind, e.g. 'Rhodes Island, Faction'.",
    },
    "entity.preview.open": {
        text: "Open page",
        description: "Footer hint in an entity's hover card when clicking the tile opens the entity's own page. Rendered uppercase.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
