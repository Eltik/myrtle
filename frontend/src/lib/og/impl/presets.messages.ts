import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "meta";

/**
 * Text rasterised into the default social card, one entry per page in
 * `presets.ts`, plus the section chips the card always draws.
 *
 * These are page metadata like the `seo()` titles - same namespace - but they
 * are a separate surface with its own budget: the card gives the title one
 * 96px line and the subtitle two at 28px, so an OG subtitle is usually a
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
        text: "Search Doctors",
        description: "Headline on the player search social card. `Doctors` is what Arknights calls its players.",
    },
    "og.userSearch.subtitle": {
        text: "Find and browse user profiles.",
        description: "Subtitle on the player search social card.",
    },

    "og.userLeaderboard.title": {
        text: "Doctor Leaderboard",
        description: "Headline on the player leaderboard social card.",
    },
    "og.userLeaderboard.subtitle": {
        text: "Top Doctors ranked by score across servers.",
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

    // The five section chips drawn across the bottom of every default card,
    // one of them highlighted by the preset's `activeTag`. Uppercased by the
    // template, so translate them in their natural case.
    "og.tag.home": {
        text: "Home",
        description: "Section chip on the default social card. Drawn uppercase, at 13px, in a row of five - keep it to one short word.",
    },
    "og.tag.collection": {
        text: "Collection",
        description: "Section chip on the default social card covering operators, enemies and stages. Drawn uppercase, at 13px, in a row of five - keep it to one short word.",
    },
    "og.tag.players": {
        text: "Players",
        description: "Section chip on the default social card covering profiles, search and the leaderboard. Drawn uppercase, at 13px, in a row of five - keep it to one short word.",
    },
    "og.tag.gacha": {
        text: "Gacha",
        description: "Section chip on the default social card. `Gacha` is the standard loanword for the game's randomised pulls. Drawn uppercase, at 13px, in a row of five - keep it to one short word.",
    },
    "og.tag.tools": {
        text: "Tools",
        description: "Section chip on the default social card covering the calculators and planners. Drawn uppercase, at 13px, in a row of five - keep it to one short word.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages, dynamic: true });
