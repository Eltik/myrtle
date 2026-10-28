import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tools";

export const messages = {
    "randomizer.ops.class": {
        text: "Class",
        description: "Heading over the class toggles. The class names themselves are game vocabulary.",
    },
    "randomizer.ops.class.aria": {
        text: "Allowed classes",
        description: "Accessible name of the group of class toggles.",
    },
    "randomizer.ops.rarity": {
        text: "Rarity",
        description: "Heading over the star-rating toggles.",
    },
    "randomizer.ops.rarity.aria": {
        text: "Allowed rarities",
        description: "Accessible name of the group of star-rating toggles.",
    },
    "randomizer.ops.rarityChip": {
        text: "{rarity}★",
        description: "One star-rating toggle, e.g. '5★'.",
    },
    "randomizer.ops.rarityChip.aria": {
        text: "{rarity} star",
        description: "Accessible name of one star-rating toggle, e.g. '5 star'. It reads as a rating, not a count, so it stays singular.",
    },
    "randomizer.ops.source": {
        text: "Source",
        description: "Heading over the toggles that narrow the pool by how an operator is obtained.",
    },
    "randomizer.ops.source.aria": {
        text: "Operator source",
        description: "Accessible name of the group of source toggles.",
    },
    "randomizer.ops.source.any": {
        text: "Any",
        description: "Source toggle: every operator, however it is obtained.",
    },
    "randomizer.ops.source.gachaless": {
        text: "Gachaless",
        description: "Source toggle: operators a player can get without headhunting (the game's paid gacha), that is welfare operators plus the ones in the recruitment pool. 'Gachaless' is the community's name for an account that never headhunts.",
    },
    "randomizer.ops.source.welfare": {
        text: "Welfare",
        description: "Source toggle: operators the game gives away (event rewards, certificate shops, Integrated Strategies, credit store, story).",
    },
    "randomizer.ops.source.recruitment": {
        text: "Recruitment",
        description: "Source toggle: operators that recruitment (the free permit-based draw with tags, not headhunting) can give.",
    },
    "randomizer.ops.squadSize": {
        text: "Squad size · {size}",
        description: "Heading over the squad-size slider, with the current value after a middle dot.",
    },
    "randomizer.ops.rules": {
        text: "Rules",
        description: "Heading over the on/off switches for how the draw behaves.",
    },
    "randomizer.ops.allowDuplicates": {
        text: "Allow duplicates",
        description: "Switch label: the same operator may be drawn more than once.",
    },
    "randomizer.ops.allowDuplicates.desc": {
        text: "Same operator can appear twice in a squad.",
        description: "Explanation under the 'Allow duplicates' switch.",
    },
    "randomizer.ops.hideUnplayable": {
        text: "Hide unplayable operators",
        description: "Switch label: leave out units a player cannot actually deploy.",
    },
    "randomizer.ops.hideUnplayable.desc": {
        text: "Exclude tokens, support-only, and reserve operators.",
        description: "Explanation under the 'Hide unplayable operators' switch. 'Token', 'support' and 'reserve' are the game's own unit categories.",
    },
    "randomizer.ops.onlyOwned": {
        text: "Only operators I own",
        description: "Switch label: draw only from the player's own roster.",
    },
    "randomizer.ops.onlyOwned.desc": {
        text: "Restrict to your roster from the linked profile.",
        description: "Explanation under the 'Only operators I own' switch; the profile is the game account linked to this site.",
    },
    "randomizer.ops.onlyE2": {
        text: "E2 only",
        description: "Switch label: only fully promoted operators. 'E2' is the game's second Elite promotion.",
    },
    "randomizer.ops.onlyE2.desc": {
        text: "Restrict further to operators at elite 2 in your roster.",
        description: "Explanation under the 'E2 only' switch. 'Elite 2' is the game's second promotion.",
    },
    "randomizer.ops.e2SixStarOnly": {
        text: "6★ only",
        description: "Switch label under 'E2 only': the E2 requirement binds 6-star operators only.",
    },
    "randomizer.ops.e2SixStarOnly.desc": {
        text: "Only require Elite 2 for 6★ operators.",
        description: "Explanation under the '6★ only' switch; lower rarities then pass at any promotion. 'Elite 2' is the game's second promotion.",
    },
    "randomizer.ops.tierList": {
        text: "Tier list",
        description: "Heading over the tier-list filter.",
    },
    "randomizer.ops.tierList.title": {
        text: "Official tier list",
        description: "Label of the dropdown that picks one of the site's official tier lists to filter by. Also its accessible name.",
    },
    "randomizer.ops.tierList.desc": {
        text: "Draw only from the tiers you keep selected.",
        description: "Explanation next to the tier-list dropdown; the tiers are the chips that appear once a list is picked.",
    },
    "randomizer.ops.tierList.none": {
        text: "None",
        description: "Dropdown option: no tier list, so the filter is off.",
    },
    "randomizer.ops.tierList.loading": {
        text: "Loading tiers...",
        description: "Shown while the chosen tier list loads.",
    },
    "randomizer.ops.tierList.unranked": {
        text: "Unranked",
        description: "Tier chip for operators the chosen tier list does not place.",
    },
    "randomizer.ops.tierList.aria": {
        text: "Allowed tiers",
        description: "Accessible name of the group of tier chips.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
