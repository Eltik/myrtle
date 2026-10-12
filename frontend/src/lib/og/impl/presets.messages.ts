import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "meta";

/**
 * Text rasterised into the default social card, one entry per page in
 * `presets.ts`, plus the section names its kicker draws.
 *
 * These are page metadata like the `seo()` titles, in the same namespace, but
 * they are a separate surface with its own budget: the card gives the title up to
 * two lines at 56 to 88px and the subtitle two at 26px, so an OG subtitle is usually a
 * shorter rewrite of the page's meta description rather than the same
 * sentence. They get their own keys so the two can be tuned apart.
 *
 * `dynamic: true`: `presets.ts` is a data table of `titleKey`/`subtitleKey`,
 * resolved through a variable, which the extractor's literal-call-site scan
 * cannot see.
 */
export const messages = {
    "og.root.title": {
        text: "Operator data, rosters, and tier lists.",
        description: "Headline on the site-wide fallback social card, used for any page with no card of its own.",
    },

    "og.home.title": {
        text: "Operator data, rosters, and tier lists.",
        description: "Headline on the landing page's social card.",
    },
    "og.home.subtitle": {
        text: "Track your roster, scout community pulls, and explore every operator.",
        description: "Subtitle under the headline on the landing page's social card.",
    },

    "og.login.title": {
        text: "Log in to Myrtle",
        description: "Headline on the sign-in social card.",
    },
    "og.login.subtitle": {
        text: "Sign in to track your roster, gacha history, and tier lists.",
        description: "Subtitle on the sign-in social card.",
    },

    "og.settings.title": {
        text: "Settings",
        description: "Headline on the account settings social card.",
    },
    "og.settings.subtitle": {
        text: "Manage your account, profile, and preferences.",
        description: "Subtitle on the account settings social card.",
    },

    "og.operators.title": {
        text: "Operators",
        description: "Headline on the operator index social card.",
    },
    "og.operators.subtitle": {
        text: "Every operator released in Arknights.",
        description: "Subtitle on the operator index social card.",
    },

    "og.enemies.title": {
        text: "Enemy Database",
        description: "Headline on the enemy index social card.",
    },
    "og.enemies.subtitle": {
        text: "Every enemy catalogued in Arknights.",
        description: "Subtitle on the enemy index social card.",
    },

    "og.stages.title": {
        text: "Stage Index",
        description: "Headline on the stage index social card.",
    },
    "og.stages.subtitle": {
        text: "Every Arknights stage, mapped with an enemy-pathing simulator.",
        description: "Subtitle on the stage index social card.",
    },

    "og.stats.title": {
        text: "Stats",
        description: "Headline on the site-wide index counts social card.",
    },
    "og.stats.subtitle": {
        text: "A live count of every operator, tier list, and roster indexed on myrtle.moe.",
        description: "Subtitle on the site-wide index counts social card.",
    },

    "og.userSearch.title": {
        text: "Search Players",
        description: "Headline on the player search social card. `Doctors` is what Arknights calls its players.",
    },
    "og.userSearch.subtitle": {
        text: "Find and browse user profiles.",
        description: "Subtitle on the player search social card.",
    },

    "og.userLeaderboard.title": {
        text: "Player Leaderboard",
        description: "Headline on the player leaderboard social card.",
    },
    "og.userLeaderboard.subtitle": {
        text: "Top players ranked by score across servers.",
        description: "Subtitle on the player leaderboard social card.",
    },

    "og.gachaCommunity.title": {
        text: "Gacha · Community",
        description: "Headline on the community gacha statistics social card. The separator is a middle dot.",
    },
    "og.gachaCommunity.subtitle": {
        text: "Community-wide pull rates, top operators, and pull timing.",
        description: "Subtitle on the community gacha statistics social card.",
    },

    "og.gachaHistory.title": {
        text: "Gacha · My History",
        description: "Headline on the personal gacha history social card. The separator is a middle dot.",
    },
    "og.gachaHistory.subtitle": {
        text: "Your synced pulls, rarity splits, and pity counters.",
        description: "Subtitle on the personal gacha history social card.",
    },

    "og.toolsDps.title": {
        text: "DPS Calculator",
        description: "Headline on the DPS calculator social card.",
    },
    "og.toolsDps.subtitle": {
        text: "Compare Arknights operator DPS.",
        description: "Subtitle on the DPS calculator social card.",
    },

    "og.toolsHps.title": {
        text: "HPS Calculator",
        description: "Headline on the healing calculator social card.",
    },
    "og.toolsHps.subtitle": {
        text: "Compare Arknights healer HPS.",
        description: "Subtitle on the healing calculator social card.",
    },

    "og.toolsSquadBuilder.title": {
        text: "Squad Builder",
        description: "Headline on the squad builder social card.",
    },
    "og.toolsSquadBuilder.subtitle": {
        text: "Build an Arknights squad and see what it covers.",
        description: "Subtitle on the squad builder social card.",
    },

    "og.toolsRecruitment.title": {
        text: "Recruitment Calculator",
        description: "Headline on the recruitment tag calculator social card.",
    },
    "og.toolsRecruitment.subtitle": {
        text: "Calculate recruitment combos.",
        description: "Subtitle on the recruitment tag calculator social card.",
    },

    "og.toolsPlanner.title": {
        text: "Operator Planner",
        description: "Headline on the operator upgrade planner social card.",
    },
    "og.toolsPlanner.subtitle": {
        text: "Plan your operator promotions, skills, and module targets.",
        description: "Subtitle on the operator upgrade planner social card.",
    },

    "og.toolsRandomizer.title": {
        text: "Randomizer",
        description: "Headline on the random stage/squad roller social card.",
    },
    "og.toolsRandomizer.subtitle": {
        text: "Roll a random stage, squad, and modifier for your next run.",
        description: "Subtitle on the random stage/squad roller social card.",
    },

    "og.toolsBirthdays.title": {
        text: "Birthdays",
        description: "Headline on the operator birthday calendar social card.",
    },
    "og.toolsBirthdays.subtitle": {
        text: "View and track Arknights operator birthdays.",
        description: "Subtitle on the operator birthday calendar social card.",
    },

    "og.toolsRelease.title": {
        text: "Release Planner",
        description: "Headline on the CN-to-EN release tracker social card.",
    },
    "og.toolsRelease.subtitle": {
        text: "When CN events, skins, and banners land on EN.",
        description: "Subtitle on the CN-to-EN release tracker social card. CN and EN are the Chinese and English game servers.",
    },

    "og.tierLists.title": {
        text: "Tier Lists",
        description: "Headline on the tier-list browser social card.",
    },
    "og.tierLists.subtitle": {
        text: "Browse official and community tier lists.",
        description: "Subtitle on the tier-list browser social card.",
    },

    "og.grids.title": {
        text: "Grids",
        description: "Headline on the grid browser social card. A grid is a board of labelled cells filled with operators and other picks.",
    },
    "og.grids.subtitle": {
        text: "Fill a grid with your picks, share it, or start from a template.",
        description: "Subtitle on the grid browser social card.",
    },

    "og.terms.title": {
        text: "Terms of Service",
        description: "Headline on the terms of service social card.",
    },
    "og.terms.subtitle": {
        text: "Legal agreement governing your use of Myrtle.",
        description: "Subtitle on the terms of service social card.",
    },

    "og.privacy.title": {
        text: "Privacy Policy",
        description: "Headline on the privacy policy social card.",
    },
    "og.privacy.subtitle": {
        text: "How we protect and handle your personal information.",
        description: "Subtitle on the privacy policy social card.",
    },

    "og.changelog.title": {
        text: "Changelog",
        description: "Headline on the changelog social card.",
    },
    "og.changelog.subtitle": {
        text: "Every commit and change shipped to myrtle.moe.",
        description: "Subtitle on the changelog social card.",
    },

    "og.music.title": {
        text: "Music",
        description: "Headline on the music gallery social card.",
    },
    "og.music.subtitle": {
        text: "Every Monster Siren album, with covers, artists and a player.",
        description: "Subtitle on the music gallery social card.",
    },

    "og.stories.title": {
        text: "Story Reader",
        description: "Headline on the story library social card.",
    },
    "og.stories.subtitle": {
        text: "Read every main story, event and operator record as the game stages it.",
        description: "Subtitle on the story library social card.",
    },

    "og.archives.title": {
        text: "Archives",
        description: "Headline on the operator archives social card: profiles, records and lore.",
    },
    "og.archives.subtitle": {
        text: "Operator files, records and voice lines from the Rhodes Island archives.",
        description: "Subtitle on the operator archives social card.",
    },

    // The section the default card names in its kicker, from the preset's
    // `section`. Uppercased by the template, so translate them in their
    // natural case.
    "og.tag.home": {
        text: "Home",
        description: "Section kicker on the default social card, above the headline. Drawn uppercase in a small monospace face with wide tracking: keep it to one short word.",
    },
    "og.tag.collection": {
        text: "Collection",
        description: "Section kicker on the default social card, above the headline. Drawn uppercase in a small monospace face with wide tracking: keep it to one short word. Covers operators, enemies, stages, stories, archives and music.",
    },
    "og.tag.players": {
        text: "Players",
        description: "Section kicker on the default social card, above the headline. Drawn uppercase in a small monospace face with wide tracking: keep it to one short word. Covers profiles, search and the leaderboard.",
    },
    "og.tag.community": {
        text: "Community",
        description: "Section kicker on the default social card, above the headline. Drawn uppercase in a small monospace face with wide tracking: keep it to one short word. Covers tier lists and grids made by players.",
    },
    "og.tag.gacha": {
        text: "Gacha",
        description: "Section kicker on the default social card, above the headline. Drawn uppercase in a small monospace face with wide tracking: keep it to one short word. `Gacha` is the standard loanword for the game's randomised pulls.",
    },
    "og.tag.tools": {
        text: "Tools",
        description: "Section kicker on the default social card, above the headline. Drawn uppercase in a small monospace face with wide tracking: keep it to one short word. Covers the calculators and planners.",
    },
    "og.tag.stats": {
        text: "Stats",
        description: "Section kicker on the default social card, above the headline. Drawn uppercase in a small monospace face with wide tracking: keep it to one short word. Covers the site-wide index counts.",
    },
    "og.tag.account": {
        text: "Account",
        description: "Section kicker on the default social card, above the headline. Drawn uppercase in a small monospace face with wide tracking: keep it to one short word. Covers sign-in and settings.",
    },
    "og.tag.legal": {
        text: "Legal",
        description: "Section kicker on the default social card, above the headline. Drawn uppercase in a small monospace face with wide tracking: keep it to one short word. Covers the terms of service and privacy policy.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages, dynamic: true });
