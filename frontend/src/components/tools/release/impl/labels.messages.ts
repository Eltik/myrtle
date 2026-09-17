import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tools";

/**
 * Player-facing names for the raw gamedata tags the planner shows on events and
 * banners. Every one of these used to reach the screen through `humanizeTag`,
 * which only lowercases and swaps underscores for spaces, so the tag strip read
 * "TYPE ACTIVITYSIDESTORY", "TYPE ACT9D0" and "MULTIPLAY V3".
 */
export const messages = {
    "release.tag.sideStory": {
        text: "Side story",
        description: "Event category: a self-contained story event. The game's own term.",
    },
    "release.tag.miniStory": {
        text: "Mini story",
        description: "Event category: a short story event, smaller than a side story. The game's own term.",
    },
    "release.tag.intermezzi": {
        text: "Intermezzi",
        description: "Event category: a story event tied to the main plot. The game's own term; leave untranslated where the game does.",
    },
    "release.tag.vignette": {
        text: "Vignette",
        description: "Event category: a very short story event. The game's own term.",
    },
    "release.tag.mainStory": {
        text: "Main story",
        description: "Event category: a main-plot chapter.",
    },
    "release.tag.rerun": {
        text: "Rerun",
        description: "Event category: a rebroadcast of an event that already ran.",
    },
    "release.tag.rogueLike": {
        text: "Integrated Strategies",
        description: "Event category: the game's roguelike mode. Its official English name.",
    },
    "release.tag.sandbox": {
        text: "Reclamation Algorithm",
        description: "Event category: the game's survival/sandbox mode. Its official English name.",
    },
    "release.tag.contingency": {
        text: "Contingency Contract",
        description: "Event category: the game's competitive risk-modifier mode. Its official English name.",
    },
    "release.tag.vectorBreak": {
        text: "Vector Breakthrough",
        description: "Event category: a one-off competitive mode. Its official English name.",
    },
    "release.tag.bossRush": {
        text: "Boss Rush",
        description: "Event category: a one-off boss-gauntlet mode. Its official English name.",
    },
    "release.tag.enemyDuel": {
        text: "Enemy Duel",
        description: "Event category: a one-off competitive mode. Its official English name.",
    },
    "release.tag.autoChess": {
        text: "Auto-chess",
        description: "Event category: a one-off auto-battler mode.",
    },
    "release.tag.arcade": {
        text: "Arcade",
        description: "Event category: a one-off minigame mode.",
    },
    "release.tag.minigame": {
        text: "Minigame",
        description: "Event category: a one-off minigame not covered by a more specific name.",
    },
    "release.tag.collection": {
        text: "Collection",
        description: "Event category: a collection/gallery event.",
    },
    "release.tag.signIn": {
        text: "Sign-in",
        description: "Event category: a daily login-reward campaign.",
    },
    "release.tag.campaign": {
        text: "Campaign",
        description: "Event category: a login/anniversary style campaign with no stages.",
    },
    "release.tag.newSkins": {
        text: "New skins",
        description: "Calendar tag on a day that introduces new operator outfits.",
    },
    "release.tag.fashionReview": {
        text: "Fashion review",
        description: "Calendar tag for the in-game skin-showcase segment. The game's own term.",
    },
    "release.tag.banner.limited": {
        text: "Limited",
        description: "Banner category: a limited headhunting banner.",
    },
    "release.tag.banner.single": {
        text: "Single rate-up",
        description: "Banner category: one boosted 6-star. 'Rate-up' is the game's term for a boosted pull chance.",
    },
    "release.tag.banner.double": {
        text: "Double rate-up",
        description: "Banner category: two boosted 6-stars.",
    },
    "release.tag.banner.normal": {
        text: "Standard",
        description: "Banner category: the permanent headhunting pool.",
    },
    "release.tag.banner.linkage": {
        text: "Collaboration",
        description: "Banner category: a crossover banner. The game's own English name for LINKAGE.",
    },
    "release.tag.banner.classic": {
        text: "Kernel",
        description: "Banner category: the older-operator pool. The game's own English name for CLASSIC.",
    },
    "release.tag.banner.attain": {
        text: "Special Headhunting",
        description: "Banner category: a guaranteed-pick banner. The game's own English name for ATTAIN.",
    },
    "release.tag.banner.classicAttain": {
        text: "Kernel Special Headhunting",
        description: "Banner category: a guaranteed-pick banner drawn from the older-operator pool.",
    },
    "release.tag.banner.fesClassic": {
        text: "Anniversary Kernel",
        description: "Banner category: the anniversary run of the older-operator pool.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
