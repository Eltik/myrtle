import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

// Every per-kind string, one block per kind in `ALL_ENTITY_KINDS` order. A new kind adds its block here and its entry in `KIND_DEFINITIONS` (kinds.tsx).

export const namespace = "tierLists";

export const messages = {
    // Shared by every kind, or by several: tile names, the drop area, the rarity, class and type filters.
    "entity.tile.label": {
        text: "{name}, {kind}",
        description: "Accessible name of a non-operator tile: the entity's name from the game data, then its kind, e.g. 'Rhodes Island, Faction'.",
    },
    "entity.tile.labelOwned": {
        text: "{name} ({owner}), {kind}",
        description: "Accessible name of a skin, module or skill tile: its name from the game data, the operator it belongs to in parentheses, then its kind, e.g. 'Stick and Sack (Hoshiguma), Module'.",
    },
    "entity.preview.open": {
        text: "Open page",
        description: "Footer hint in an entity's hover card when clicking the tile opens the entity's own page. Rendered uppercase.",
    },
    "edit.pool.dropArea.any": {
        text: "Drag a tile here to unplace it",
        description: "Accessible name of the pool area for any kind other than operators; the area doubles as the target for taking a tile off the board.",
    },
    "edit.pool.rarity": {
        text: "Rarity",
        description: "Label of the rarity filter in the pool dialog. Rendered uppercase.",
    },
    "edit.pool.rarity.group": {
        text: "Filter by rarity",
        description: "Accessible name of the row of rarity filter buttons.",
    },
    "edit.pool.rarity.option": {
        text: "{rarity} star",
        description: "Accessible name of one rarity filter button, e.g. '6 star'.",
    },
    "edit.pool.class": {
        text: "Class",
        description: "Label of the class filter in the pool dialog. Rendered uppercase.",
    },
    "edit.pool.class.group": {
        text: "Filter by class",
        description: "Accessible name of the row of class filter buttons.",
    },
    "edit.pool.type": {
        text: "Type",
        description: "Label of the type filter (event category, or Stronghold bond type) in the pool dialog. Rendered uppercase.",
    },
    "edit.pool.type.group": {
        text: "Filter by type",
        description: "Accessible name of the row of type filter buttons.",
    },

    "entity.kinds.operator": {
        text: "Operators",
        description: "Name of a kind of thing a tier list can rank: playable characters. Plural. Used on the editor's pool tabs and in the list of kinds a list offers.",
    },
    "entity.kind.operator": {
        text: "Operator",
        description: "Singular kind label shown small above an entity's name in its hover card.",
    },
    "edit.kinds.desc.operator": {
        text: "Playable characters",
        description: "One-line explanation under 'Operators' in the kinds dialog.",
    },
    "edit.pool.kicker": {
        text: "Operator pool",
        description: "Small uppercase label above the list of operators that can be dragged onto a tier.",
    },
    "edit.pool.dialogTitle": {
        text: "Operator pool",
        description: "Title of the larger operator-pool dialog.",
    },
    "edit.pool.search.label": {
        text: "Search operators",
        description: "Accessible name of the pool's search box.",
    },
    "edit.pool.gridLabel": {
        text: "Available operators",
        description: "Accessible name of the grid of operator tiles in the pool.",
    },
    "edit.pool.dropArea": {
        text: "Drag operators here to unplace them",
        description: "Accessible name of the pool area, which doubles as the target for taking an operator off the board.",
    },

    "entity.kinds.skill": {
        text: "Skills",
        description: "Name of a kind of thing a tier list can rank: operator skills, one per operator and skill slot. Plural. Used on pool tabs and in the kinds settings.",
    },
    "entity.kind.skill": {
        text: "Skill",
        description: "Singular kind label shown small above a skill's name in its hover card.",
    },
    "edit.kinds.desc.skill": {
        text: "Each operator's skills, slot by slot",
        description: "One-line explanation under 'Skills' in the kinds dialog. A skill shared by several operators is one entry per operator.",
    },
    "edit.pool.kicker.skill": {
        text: "Skill pool",
        description: "Small uppercase label above the pool of operator skills that can be dragged onto a tier; also the title of the larger pool dialog.",
    },
    "edit.pool.search.skill": {
        text: "Search skills",
        description: "Accessible name of the pool's search box while it lists skills. It matches the skill and the operator.",
    },
    "edit.pool.grid.skill": {
        text: "Available skills",
        description: "Accessible name of the grid of skill tiles in the pool.",
    },
    "entity.skill.slot": {
        text: "Skill {slot}",
        description: "Which of an operator's skill slots a skill fills, e.g. 'Skill 2'. Shown in hover cards. {slot} is a number from 1 to 3.",
    },
    "edit.pool.slot": {
        text: "Slot",
        description: "Label of the filter by skill slot (S1, S2, S3) in the pool dialog. Rendered uppercase.",
    },
    "edit.pool.slot.group": {
        text: "Filter by skill slot",
        description: "Accessible name of the row of skill-slot filter buttons.",
    },
    "edit.pool.sp": {
        text: "SP",
        description: "Label of the filter by how a skill recovers SP (over time, on attack, when hit) in the pool dialog. Rendered uppercase.",
    },
    "edit.pool.sp.group": {
        text: "Filter by SP recovery",
        description: "Accessible name of the row of SP-recovery filter buttons.",
    },
    "edit.pool.sp.auto": {
        text: "Auto",
        description: "SP-recovery filter option: the skill charges over time.",
    },
    "edit.pool.sp.offensive": {
        text: "Offensive",
        description: "SP-recovery filter option: the skill charges when the operator attacks.",
    },
    "edit.pool.sp.defensive": {
        text: "Defensive",
        description: "SP-recovery filter option: the skill charges when the operator is hit.",
    },
    "edit.pool.sp.passive": {
        text: "Passive",
        description: "SP-recovery filter option: a passive skill, which has no SP to recover.",
    },

    "entity.kinds.module": {
        text: "Modules",
        description: "Name of a kind of thing a tier list can rank: operator modules (equipment that adds an X, Y or other branch to an operator). Plural. Used on pool tabs and in the kinds settings.",
    },
    "entity.kind.module": {
        text: "Module",
        description: "Singular kind label shown small above a module's name in its hover card.",
    },
    "edit.kinds.desc.module": {
        text: "Every operator module",
        description: "One-line explanation under 'Modules' in the kinds dialog.",
    },
    "edit.pool.kicker.module": {
        text: "Module pool",
        description: "Small uppercase label above the pool of operator modules that can be dragged onto a tier; also the title of the larger pool dialog.",
    },
    "edit.pool.search.module": {
        text: "Search modules",
        description: "Accessible name of the pool's search box while it lists modules. It matches the module and the operator.",
    },
    "edit.pool.grid.module": {
        text: "Available modules",
        description: "Accessible name of the grid of module tiles in the pool.",
    },

    "entity.kinds.skin": {
        text: "Skins",
        description: "Name of a kind of thing a tier list can rank: operator outfits from the store and event rewards. Plural. Used on pool tabs and in the kinds settings.",
    },
    "entity.kind.skin": {
        text: "Skin",
        description: "Singular kind label shown small above an outfit's name in its hover card.",
    },
    "edit.kinds.desc.skin": {
        text: "Outfits from the store and event rewards",
        description: "One-line explanation under 'Skins' in the kinds dialog. Default and Elite 2 art are not offered.",
    },
    "edit.pool.kicker.skin": {
        text: "Skin pool",
        description: "Small uppercase label above the pool of operator outfits that can be dragged onto a tier; also the title of the larger pool dialog.",
    },
    "edit.pool.search.skin": {
        text: "Search skins",
        description: "Accessible name of the pool's search box while it lists operator outfits. It matches the outfit, the operator and the brand.",
    },
    "edit.pool.grid.skin": {
        text: "Available skins",
        description: "Accessible name of the grid of outfit tiles in the pool.",
    },
    "edit.pool.brand": {
        text: "Brand",
        description: "Label of the filter by outfit brand (the store line an outfit belongs to, such as EPOQUE) in the pool dialog. Rendered uppercase.",
    },
    "edit.pool.brand.group": {
        text: "Filter by brand",
        description: "Accessible name of the row of outfit-brand filter buttons.",
    },

    "entity.kinds.class": {
        text: "Classes",
        description: "Name of a kind of thing a tier list can rank: the eight operator classes (Vanguard, Guard, and so on). Plural. Used on pool tabs and in the kinds settings.",
    },
    "entity.kind.class": {
        text: "Class",
        description: "Singular kind label shown small above a class's name in its hover card.",
    },
    "edit.kinds.desc.class": {
        text: "The eight classes, Vanguard to Specialist",
        description: "One-line explanation under 'Classes' in the kinds dialog.",
    },
    "edit.pool.kicker.class": {
        text: "Class pool",
        description: "Small uppercase label above the pool of classes that can be dragged onto a tier; also the title of the larger pool dialog.",
    },
    "edit.pool.search.class": {
        text: "Search classes",
        description: "Accessible name of the pool's search box while it lists classes.",
    },
    "edit.pool.grid.class": {
        text: "Available classes",
        description: "Accessible name of the grid of classes tiles in the pool.",
    },

    "entity.kinds.subclass": {
        text: "Subclasses",
        description: "Name of a kind of thing a tier list can rank: operator subclasses (archetypes such as Charger or Lord). Plural. Used on pool tabs and in the kinds settings.",
    },
    "entity.kind.subclass": {
        text: "Subclass",
        description: "Singular kind label shown small above a subclass's name in its hover card.",
    },
    "edit.kinds.desc.subclass": {
        text: "Archetypes within a class",
        description: "One-line explanation under 'Subclasses' in the kinds dialog.",
    },
    "edit.pool.kicker.subclass": {
        text: "Subclass pool",
        description: "Small uppercase label above the pool of subclasses that can be dragged onto a tier; also the title of the larger pool dialog.",
    },
    "edit.pool.search.subclass": {
        text: "Search subclasses",
        description: "Accessible name of the pool's search box while it lists subclasses.",
    },
    "edit.pool.grid.subclass": {
        text: "Available subclasses",
        description: "Accessible name of the grid of subclasses tiles in the pool.",
    },

    "entity.kinds.faction": {
        text: "Factions",
        description: "Name of a kind of thing a tier list can rank: nations, groups and teams operators belong to. Plural. Used on pool tabs and in the kinds settings.",
    },
    "entity.kind.faction": {
        text: "Faction",
        description: "Singular kind label shown small above a faction's name in its hover card.",
    },
    "edit.kinds.desc.faction": {
        text: "Nations, groups and teams",
        description: "One-line explanation under 'Factions' in the kinds dialog.",
    },
    "edit.pool.kicker.faction": {
        text: "Faction pool",
        description: "Small uppercase label above the pool of factions that can be dragged onto a tier; also the title of the larger pool dialog.",
    },
    "edit.pool.search.faction": {
        text: "Search factions",
        description: "Accessible name of the pool's search box while it lists factions.",
    },
    "edit.pool.grid.faction": {
        text: "Available factions",
        description: "Accessible name of the grid of factions tiles in the pool.",
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
    "edit.pool.level": {
        text: "Level",
        description: "Label of the faction-level filter (nation, group, team) in the pool dialog. Rendered uppercase.",
    },
    "edit.pool.level.group": {
        text: "Filter by level",
        description: "Accessible name of the row of faction-level filter buttons.",
    },

    "entity.kinds.enemy": {
        text: "Enemies",
        description: "Name of a kind of thing a tier list can rank: enemies from the game's enemy handbook. Plural. Used on pool tabs and in the kinds settings.",
    },
    "entity.kind.enemy": {
        text: "Enemy",
        description: "Singular kind label shown small above an enemy's name in its hover card.",
    },
    "edit.kinds.desc.enemy": {
        text: "Every enemy in the handbook",
        description: "One-line explanation under 'Enemies' in the kinds dialog.",
    },
    "edit.pool.kicker.enemy": {
        text: "Enemy pool",
        description: "Small uppercase label above the pool of enemies that can be dragged onto a tier; also the title of the larger pool dialog.",
    },
    "edit.pool.search.enemy": {
        text: "Search enemies",
        description: "Accessible name of the pool's search box while it lists enemies.",
    },
    "edit.pool.grid.enemy": {
        text: "Available enemies",
        description: "Accessible name of the grid of enemies tiles in the pool.",
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
    "edit.pool.rank": {
        text: "Rank",
        description: "Label of the enemy-rank filter (normal, elite, leader) in the pool dialog. Rendered uppercase.",
    },
    "edit.pool.rank.group": {
        text: "Filter by rank",
        description: "Accessible name of the row of enemy-rank filter buttons.",
    },

    "entity.kinds.event": {
        text: "Events",
        description: "Name of a kind of thing a tier list can rank: in-game events (side stories, vignettes and the like). Plural. Used on pool tabs and in the kinds settings.",
    },
    "entity.kind.event": {
        text: "Event",
        description: "Singular kind label shown small above an event's name in its hover card.",
    },
    "edit.kinds.desc.event": {
        text: "Side stories, vignettes and other events",
        description: "One-line explanation under 'Events' in the kinds dialog.",
    },
    "edit.pool.kicker.event": {
        text: "Event pool",
        description: "Small uppercase label above the pool of events that can be dragged onto a tier; also the title of the larger pool dialog.",
    },
    "edit.pool.search.event": {
        text: "Search events",
        description: "Accessible name of the pool's search box while it lists events.",
    },
    "edit.pool.grid.event": {
        text: "Available events",
        description: "Accessible name of the grid of events tiles in the pool.",
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
    "edit.pool.edition": {
        text: "Edition",
        description: "Label of the filter that tells an event's first run from its reruns, in the pool dialog. Rendered uppercase.",
    },
    "edit.pool.edition.group": {
        text: "Filter by edition",
        description: "Accessible name of the row of edition filter buttons.",
    },
    "edit.pool.edition.original": {
        text: "Original",
        description: "Edition filter option: the event's first run, as opposed to a rerun.",
    },

    "entity.kinds.main_story": {
        text: "Main story",
        description: "Name of a kind of thing a tier list can rank: the episodes of the game's main story. Used on pool tabs and in the kinds settings.",
    },
    "entity.kind.main_story": {
        text: "Main story episode",
        description: "Singular kind label shown small above a main story episode's name in its hover card.",
    },
    "edit.kinds.desc.main_story": {
        text: "Every episode of the main story",
        description: "One-line explanation under 'Main story' in the kinds dialog.",
    },
    "edit.pool.kicker.main_story": {
        text: "Main story pool",
        description: "Small uppercase label above the pool of main story episodes that can be dragged onto a tier; also the title of the larger pool dialog.",
    },
    "edit.pool.search.main_story": {
        text: "Search main story episodes",
        description: "Accessible name of the pool's search box while it lists main story episodes.",
    },
    "edit.pool.grid.main_story": {
        text: "Available main story episodes",
        description: "Accessible name of the grid of main story episode tiles in the pool.",
    },
    "entity.mainStory.episode": {
        text: "Episode {number}",
        description: "A main story episode by its number, as the game prints it (Episode 14). Used in hover cards and matched by the pool search. {number} is a number.",
    },
    "edit.pool.act": {
        text: "Act",
        description: "Label of the filter by act (the game's groups of main story episodes, such as Hour of An Awakening) in the pool dialog. Rendered uppercase.",
    },
    "edit.pool.act.group": {
        text: "Filter by act",
        description: "Accessible name of the row of main story act filter buttons.",
    },

    "entity.kinds.integrated_strategies": {
        text: "Integrated Strategies",
        description: "Name of a kind of thing a tier list can rank: the game's roguelike mode, its themes and their items (collectibles, squads, Foldartals and the like). Keep the game's own name for the mode. Used on pool tabs and in the kinds settings.",
    },
    "entity.kind.integrated_strategies": {
        text: "Integrated Strategies",
        description: "Singular kind label shown small above the name of an Integrated Strategies theme or item in its hover card. Keep the game's own name for the mode.",
    },
    "edit.kinds.desc.integrated_strategies": {
        text: "Themes, collectibles, squads and more",
        description: "One-line explanation under 'Integrated Strategies' in the kinds dialog: the roguelike mode's themes and the items collected in them.",
    },
    "edit.pool.kicker.integrated_strategies": {
        text: "Integrated Strategies pool",
        description: "Small uppercase label above the pool of Integrated Strategies themes and items that can be dragged onto a tier; also the title of the larger pool dialog. Keep the game's own name for the mode.",
    },
    "edit.pool.search.integrated_strategies": {
        text: "Search Integrated Strategies",
        description: "Accessible name of the pool's search box while it lists Integrated Strategies themes and items.",
    },
    "edit.pool.grid.integrated_strategies": {
        text: "Available Integrated Strategies entries",
        description: "Accessible name of the grid of Integrated Strategies theme and item tiles in the pool.",
    },
    "entity.isTheme": {
        text: "IS{number}",
        description: "Short name of one Integrated Strategies theme by its number, e.g. 'IS3' for Mizuki & Caerula Arbor. Used as a pool filter and in hover cards. {number} is a number.",
    },
    "entity.isItem.theme": {
        text: "Theme",
        description: "Integrated Strategies entry type: the theme (season) itself rather than an item in it. Used as a pool filter.",
    },
    "entity.isItem.band": {
        text: "Squad",
        description: "Integrated Strategies item type: a starting squad. Used as a pool filter and in hover cards.",
    },
    "entity.isItem.relic": {
        text: "Collectible",
        description: "Integrated Strategies item type: a collectible (relic) found during a run. The English game calls these Collectibles. Used as a pool filter and in hover cards.",
    },
    "entity.isItem.active_tool": {
        text: "Tool",
        description: "Integrated Strategies item type: a deployable support tool. Used as a pool filter and in hover cards.",
    },
    "entity.isItem.explore_tool": {
        text: "Exploration tool",
        description: "Integrated Strategies item type: an exploration tool from the Expeditioner's Joklumarkar theme. Used as a pool filter and in hover cards.",
    },
    "entity.isItem.capsule": {
        text: "Play",
        description: "Integrated Strategies item type: a Play performed by the troupe in the Phantom & Crimson Solitaire theme. The English game calls these Plays. Used as a pool filter and in hover cards.",
    },
    "entity.isItem.totem": {
        text: "Foldartal",
        description: "Integrated Strategies item type: a Foldartal from the Expeditioner's Joklumarkar theme. Keep the game's own term. Used as a pool filter and in hover cards.",
    },
    "entity.isItem.fragment": {
        text: "Thought",
        description: "Integrated Strategies item type: a Thought from the Sarkaz's Furnaceside Fables theme. The English game calls these Thoughts. Used as a pool filter and in hover cards.",
    },
    "entity.isItem.wrath": {
        text: "Wrath",
        description: "Integrated Strategies item type: a Wrath from the Sui's Garden of Grotesqueries theme. Used as a pool filter and in hover cards.",
    },
    "entity.isItem.copper": {
        text: "Tongbao",
        description: "Integrated Strategies item type: a Tongbao coin from the Sui's Garden of Grotesqueries theme. Keep the game's own term. Used as a pool filter and in hover cards.",
    },
    "edit.pool.theme": {
        text: "Theme",
        description: "Label of the filter by Integrated Strategies theme (IS2, IS3...) in the pool dialog. Rendered uppercase.",
    },
    "edit.pool.theme.group": {
        text: "Filter by theme",
        description: "Accessible name of the row of Integrated Strategies theme filter buttons.",
    },

    "entity.kinds.stronghold_bond": {
        text: "Stronghold bonds",
        description: "Name of a kind of thing a tier list can rank: the bonds of Stronghold Protocol, the game's auto-chess mode. Plural. Used on pool tabs and in the kinds settings.",
    },
    "entity.kind.stronghold_bond": {
        text: "Stronghold bond",
        description: "Singular kind label shown small above a Stronghold Protocol bond's name in its hover card.",
    },
    "edit.kinds.desc.stronghold_bond": {
        text: "Faction and trait bonds from Stronghold Protocol",
        description: "One-line explanation under 'Stronghold bonds' in the kinds dialog. Stronghold Protocol is the game's auto-chess mode.",
    },
    "edit.pool.kicker.stronghold_bond": {
        text: "Stronghold bond pool",
        description: "Small uppercase label above the pool of Stronghold bonds that can be dragged onto a tier; also the title of the larger pool dialog.",
    },
    "edit.pool.search.stronghold_bond": {
        text: "Search Stronghold bonds",
        description: "Accessible name of the pool's search box while it lists Stronghold bonds.",
    },
    "edit.pool.grid.stronghold_bond": {
        text: "Available Stronghold bonds",
        description: "Accessible name of the grid of Stronghold bonds tiles in the pool.",
    },
    "entity.bondType.season": {
        text: "Faction bond",
        description: "Stronghold Protocol bond type: a bond that gathers operators of some factions. Used as a pool filter and in hover cards.",
    },
    "entity.bondType.regular": {
        text: "Trait bond",
        description: "Stronghold Protocol bond type: a bond built around a shared trait rather than a faction. Used as a pool filter and in hover cards.",
    },

    "entity.kinds.story_sprite": {
        text: "Story characters",
        description: "Name of a kind of thing a tier list can rank: the character sprites drawn in the game's story scenes, operators and non-playable characters alike. Plural. Used on pool tabs and in the kinds settings.",
    },
    "entity.kind.story_sprite": {
        text: "Story character",
        description: "Singular kind label shown small above a story character's name in its hover card.",
    },
    "edit.kinds.desc.story_sprite": {
        text: "Operators and NPCs as the story draws them",
        description: "One-line explanation under 'Story characters' in the kinds dialog. NPC means a non-playable character.",
    },
    "edit.pool.kicker.story_sprite": {
        text: "Story character pool",
        description: "Small uppercase label above the pool of story character sprites that can be dragged onto a tier; also the title of the larger pool dialog.",
    },
    "edit.pool.search.story_sprite": {
        text: "Search story characters",
        description: "Accessible name of the pool's search box while it lists story characters.",
    },
    "edit.pool.grid.story_sprite": {
        text: "Available story characters",
        description: "Accessible name of the grid of story character tiles in the pool.",
    },
    "entity.spriteSource.operator": {
        text: "Operator",
        description: "Story character source: the sprite belongs to a playable operator. Used as a pool filter and in hover cards.",
    },
    "entity.spriteSource.npc": {
        text: "NPC",
        description: "Story character source: the sprite belongs to a non-playable character. Used as a pool filter and in hover cards.",
    },
    "edit.pool.source": {
        text: "Source",
        description: "Label of the filter that tells operator sprites from non-playable characters in the story character pool. Rendered uppercase.",
    },
    "edit.pool.source.group": {
        text: "Filter by source",
        description: "Accessible name of the row of story-character source filter buttons.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
