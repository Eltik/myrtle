import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

/**
 * 'Module' is the game's own equipment system and 'skin' its cosmetic outfits -
 * a translation should follow the game's wording for both.
 */
export const namespace = "user";

export const messages = {
    "profile.stats.modules.title": {
        text: "Modules",
        description: "Heading of the top half of the card, covering operator modules.",
    },
    "profile.stats.modules.unlocked": {
        text: "Unlocked",
        description: "Tile caption: modules unlocked out of those available. Rendered uppercase by CSS.",
    },
    "profile.stats.modules.unlocked.tooltip": {
        text: "{unlocked} of {total} available modules unlocked",
        description: "Tooltip on the Unlocked tile.",
    },
    "profile.stats.modules.maxLevel": {
        text: "Max Lv",
        description: "Tile caption: modules at their highest level. 'Lv' is the game's abbreviation for level; very little room.",
    },
    "profile.stats.modules.maxLevel.tooltip": {
        text: "Modules upgraded to level 3",
        description: "Tooltip on the Max Lv tile.",
    },
    "profile.stats.modules.unlockRate": {
        text: "Unlock Rate",
        description: "Label over the module-unlock bar.",
    },
    "profile.stats.modules.gap.locked": {
        text: "locked",
        description: "Remaining-work pill: modules available but not unlocked. Follows a count.",
    },
    "profile.stats.modules.gap.locked.tooltip": {
        text: "Click to view modules available on owned operators but not unlocked",
        description: "Tooltip on the 'locked' pill.",
    },
    "profile.stats.modules.gap.belowMax": {
        text: "below max",
        description: "Remaining-work pill: unlocked modules not yet at their highest level. Follows a count.",
    },
    "profile.stats.modules.gap.belowMax.tooltip": {
        text: "Click to view unlocked modules that are not yet at level 3",
        description: "Tooltip on the 'below max' pill.",
    },
    "profile.stats.skins.title": {
        text: "Skins",
        description: "Heading of the bottom half of the card, covering cosmetic outfits.",
    },
    "profile.stats.skins.collected": {
        text: "Skins Collected",
        description: "Tile caption: outfits owned out of those available. Rendered uppercase by CSS.",
    },
    "profile.stats.skins.collected.tooltip": {
        text: "{owned} of {total} non-default skins collected",
        description: "Tooltip on the Skins Collected tile. A 'default' skin is an operator's base outfit, which every account has.",
    },
    "profile.stats.skins.rate": {
        text: "Collected",
        description: "Label over the skin-collection bar.",
    },
    "profile.stats.skins.gap.missing": {
        text: "missing",
        description: "Remaining-work pill: outfits not yet owned. Follows a count.",
    },
    "profile.stats.skins.gap.missing.tooltip": {
        text: "Open the skin collection viewer",
        description: "Tooltip on the 'missing' pill, which opens the outfit browser.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
