import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "meta";

/**
 * Page titles and descriptions passed to `seo()` from every route's `head()`.
 *
 * They live here, not next to the routes, because `src/routes/**` is a file-based route tree
 * and a `*.messages.ts` dropped in there would be read as a route module. `lib/meta.ts` is the
 * one module every `head()` resolves them through, so the text sits next to it.
 *
 * `dynamic: true` is about what the extractor can see, not how keys are written (the call
 * sites are plain literals, `t("enemies.title")`): `scripts/i18n-extract.mjs` collects usage
 * from bindings initialised by `useT(...)`, and `head()` is not a component, so it resolves
 * through `metaT()`. Without the flag every key below is reported defined-but-unused and the
 * real dead keys are lost in the noise.
 */
export const messages = {
    "root.title": {
        text: "Myrtle",
        description: "Site name, used as the default document title and in the `X • Myrtle` suffix.",
    },
    "root.description": {
        text: "Arknights companion - operators, rosters, tier lists.",
        description: "Site-wide meta description, used on any page that does not set its own.",
    },

    "home.title": {
        text: "Myrtle",
        description: "Document title of the landing page. The site name on its own, deliberately.",
    },
    "home.description": {
        text: "Track your roster, scout community pulls, and explore every operator.",
        description: "Meta description of the landing page.",
    },

    "operators.title": {
        text: "Operators",
        description: "Document title of the operator index.",
    },
    "operators.description": {
        text: "View all operators released in Arknights.",
        description: "Meta description of the operator index.",
    },
    "operator.fallbackTitle": {
        text: "Operator",
        description: "Document title of an operator page whose data failed to load, so no operator name is known.",
    },

    "enemies.title": {
        text: "Enemies",
        description: "Document title of the enemy index.",
    },
    "enemies.description": {
        text: "View every enemy catalogued in Arknights.",
        description: "Meta description of the enemy index.",
    },
    "enemy.fallbackTitle": {
        text: "Enemy",
        description: "Document title of an enemy page whose data failed to load, so no enemy name is known.",
    },
    "enemy.descriptionFallback": {
        text: "Enemy {index}",
        description: "Meta description of an enemy page that has no description text of its own. `{index}` is the in-game enemy code, e.g. `B1`.",
    },

    "stories.title": {
        text: "Stories",
        description: "Document title of the story library (the Archives reader).",
    },
    "stories.description": {
        text: "Read every Arknights story: main chapters, events and operator records.",
        description: "Meta description of the story library.",
    },
    "story.title": {
        text: "{name} - Story",
        description: "Document title of one story. `{name}` is the group and story name from game data.",
    },
    "story.description": {
        text: "Read {name} in the Arknights story reader.",
        description: "Meta description of one story. `{name}` is the group and story name from game data.",
    },

    "stages.title": {
        text: "Stages",
        description: "Document title of the stage index.",
    },
    "stages.description": {
        text: "Browse every Arknights stage with an enemy pathing simulator.",
        description: "Meta description of the stage index.",
    },
    "stage.title": {
        text: "{name} - Stage",
        description: "Document title of one stage. `{name}` is the stage code, optionally followed by the stage name - both come from game data and are not translated here.",
    },
    "stage.description": {
        text: "Tile layout and enemy pathing for {name} in Arknights.",
        description: "Meta description of one stage. `{name}` is the stage code and name from game data.",
    },

    "tierLists.title": {
        text: "Tier Lists",
        description: "Document title of the tier-list browser.",
    },
    "tierLists.description": {
        text: "Browse official and community tier lists for Arknights operators.",
        description: "Meta description of the tier-list browser.",
    },
    "tierList.fallbackTitle": {
        text: "Tier List",
        description: "Document title of a tier-list page whose data failed to load, so no list title is known.",
    },
    "tierList.descriptionFallback": {
        text: "{type, select, official {Official tier list with {count} tiers on myrtle.moe.} other {Community tier list with {count} tiers on myrtle.moe.}}",
        description: "Meta description of a tier list whose author wrote none. `{type}` is `official` or `community`; `{count}` is the number of tiers and is deliberately not pluralised, matching the English source.",
    },
    "myTierLists.title": {
        text: "My Tier Lists",
        description: "Document title of the signed-in user's own tier lists.",
    },
    "myTierLists.description": {
        text: "Manage, edit, and share the tier lists you've created.",
        description: "Meta description of the signed-in user's own tier lists.",
    },
    "editTierList.title": {
        text: "Edit Tier List",
        description: "Document title of the tier-list editor.",
    },
    "editTierList.description": {
        text: "Edit your tier list.",
        description: "Meta description of the tier-list editor.",
    },
    "grids.title": {
        text: "Grids",
        description: "Document title of the grid browser. A grid is a board of labelled cells filled with operators and other picks.",
    },
    "grids.description": {
        text: "Fill a grid of Arknights prompts with your picks, share it, or use one as a template.",
        description: "Meta description of the grid browser.",
    },
    "grid.fallbackTitle": {
        text: "Grid",
        description: "Document title of a grid page whose data failed to load.",
    },
    "grid.descriptionFallback": {
        text: "A {rows} by {cols} Arknights grid by {owner} on myrtle.moe.",
        description: "Meta description of a grid whose owner wrote none. {rows} and {cols} are its size; {owner} is the owner's display name.",
    },
    "myGrids.title": {
        text: "My Grids",
        description: "Document title of the signed-in user's own grids.",
    },
    "myGrids.description": {
        text: "Create, edit, and share your grids.",
        description: "Meta description of the signed-in user's own grids.",
    },
    "editGrid.title": {
        text: "Edit Grid",
        description: "Document title of the grid editor.",
    },
    "editGrid.description": {
        text: "Edit your grid.",
        description: "Meta description of the grid editor.",
    },

    "user.fallbackTitle": {
        text: "Player",
        description: "Document title of a player profile whose data failed to load. `Doctor` is what Arknights calls the player.",
    },
    "user.description": {
        text: "{hasLevel, select, yes {{hasGrade, select, yes {Player profile • Lv {level} • {grade}} other {Player profile • Lv {level}}}} other {{hasGrade, select, yes {Player profile • {grade}} other {Player profile}}}}",
        description: "Meta description of a player profile that has no resume text. `{hasLevel}` and `{hasGrade}` are `yes` or `no` and say which of the two optional fragments the profile has; `{level}` is the account level and `{grade}` the account grade.",
    },
    "userLeaderboard.title": {
        text: "Leaderboard",
        description: "Document title of the player leaderboard.",
    },
    "userLeaderboard.description": {
        text: "Top players ranked by score across servers.",
        description: "Meta description of the player leaderboard. `Doctors` is what Arknights calls its players.",
    },
    "userSearch.title": {
        text: "Search Players",
        description: "Document title of player search. `Doctors` is what Arknights calls its players.",
    },
    "userSearch.description": {
        text: "Find and browse user profiles.",
        description: "Meta description of player search.",
    },

    "gachaCommunity.title": {
        text: "Gacha · Community",
        description: "Document title of community-wide gacha statistics. The separator is a middle dot.",
    },
    "gachaCommunity.description": {
        text: "Community-wide gacha statistics - pull rates, most-pulled operators, and pull timing across opted-in doctors.",
        description: "Meta description of community-wide gacha statistics.",
    },
    "gachaHistory.title": {
        text: "Gacha · My History",
        description: "Document title of the signed-in user's own gacha history. The separator is a middle dot.",
    },
    "gachaHistory.description": {
        text: "Your personal gacha pull history, pity counters, and operator statistics.",
        description: "Meta description of the signed-in user's own gacha history.",
    },

    "stats.title": {
        text: "Stats",
        description: "Document title of the site-wide index counts page.",
    },
    "stats.description": {
        text: "A live count of every operator, skill, module, tier list, and roster indexed on myrtle.moe.",
        description: "Meta description of the site-wide index counts page.",
    },

    "changelog.title": {
        text: "Changelog",
        description: "Document title of the changelog.",
    },
    "changelog.description": {
        text: "Every commit shipped to myrtle.moe, pulled live from GitHub and grouped by day, week, and month.",
        description: "Meta description of the changelog.",
    },

    "settings.title": {
        text: "Settings",
        description: "Document title of account settings.",
    },
    "settings.description": {
        text: "Manage your account, profile, and preferences.",
        description: "Meta description of account settings.",
    },

    "privacy.title": {
        text: "Privacy Policy",
        description: "Document title of the privacy policy.",
    },
    "privacy.description": {
        text: "Your privacy matters. Here's how Myrtle protects and handles your personal information.",
        description: "Meta description of the privacy policy.",
    },
    "terms.title": {
        text: "Terms of Service",
        description: "Document title of the terms of service.",
    },
    "terms.description": {
        text: "Legal agreement governing your use of Myrtle.",
        description: "Meta description of the terms of service.",
    },

    "toolsDps.title": {
        text: "DPS Calculator",
        description: "Document title of the damage-per-second calculator.",
    },
    "toolsDps.description": {
        text: "Compare Arknights operator DPS across varying enemy DEF, RES, or target count. Configure skills, modules, buffs, and conditionals.",
        description: "Meta description of the DPS calculator. DEF and RES are the in-game defence and magic-resistance stats.",
    },
    "toolsHps.title": {
        text: "HPS Calculator",
        description: "Document title of the healing-per-second calculator.",
    },
    "toolsHps.description": {
        text: "Compare Arknights healer HPS across target counts and team buffs. Configure skills, modules, buffs, and conditionals.",
        description: "Meta description of the HPS calculator.",
    },
    "toolsRecruitment.title": {
        text: "Recruitment Calculator",
        description: "Document title of the recruitment tag calculator.",
    },
    "toolsRecruitment.description": {
        text: "Calculate optimal tag combinations for Arknights recruitment. Find guaranteed 5-star and 6-star operators with the best tag combos.",
        description: "Meta description of the recruitment tag calculator.",
    },
    "toolsPlanner.title": {
        text: "Operator Planner",
        description: "Document title of the operator upgrade planner.",
    },
    "toolsPlanner.description": {
        text: "Plan your operator promotion, level, skill, and module targets.",
        description: "Meta description of the operator upgrade planner.",
    },
    "toolsRandomizer.title": {
        text: "Randomizer",
        description: "Document title of the random stage/squad roller.",
    },
    "toolsRandomizer.description": {
        text: "Roll a random Arknights stage, squad, and challenge modifier. Logged-in users can constrain rolls to stages they've cleared and operators they own.",
        description: "Meta description of the random stage/squad roller.",
    },
    "toolsBirthdays.title": {
        text: "Birthdays",
        description: "Document title of the operator birthday calendar.",
    },
    "toolsBirthdays.description": {
        text: "View and track Arknights operator birthdays.",
        description: "Meta description of the operator birthday calendar.",
    },
    "toolsRelease.title": {
        text: "Release Planner",
        description: "Document title of the CN-to-EN release tracker.",
    },
    "toolsRelease.description": {
        text: "When CN events, skins, and banners land on EN: confirmed, announced, or estimated with a band.",
        description: "Meta description of the CN-to-EN release tracker. CN and EN are the Chinese and English game servers.",
    },

    "admin.title": {
        text: "myrtle.moe · admin",
        description: "Document title of the internal operations console. `myrtle.moe` is the domain and stays as-is; the separator is a middle dot.",
    },
    "admin.description": {
        text: "Operations console for myrtle.moe.",
        description: "Meta description of the internal operations console.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages, dynamic: true });
