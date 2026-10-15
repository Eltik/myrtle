import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

/**
 * Labels shared by both gacha screens: the banner buckets this feature sorts
 * pulls into, a banner's run status, and the readable name it gives each
 * `gachaRuleType`. `constants.ts` is plain data with no React, so it carries
 * message KEYS and whichever component renders a row resolves it with `t()`.
 *
 * Banner and operator NAMES are absent on purpose: they come from the game
 * data, which ships its own translation per region.
 */
export const namespace = "gacha";

export const messages = {
    "banner.group.limited": {
        text: "Limited",
        description: "Banner bucket: a limited-time headhunting pool. Its pity counter does not carry over once the banner closes.",
    },
    "banner.group.linkage": {
        text: "Collab",
        description: "Banner bucket: a crossover banner run with another franchise. Short for collaboration; the space is tight.",
    },
    "banner.group.regular": {
        text: "Standard",
        description: "Banner bucket: the ordinary rotating headhunting pool.",
    },
    "banner.group.special": {
        text: "Kernel",
        description: "Banner bucket: the pool of older operators. 'Kernel' is the game's own name for it.",
    },
    "banner.group.boot": {
        text: "Starter Headhunting",
        description: "Banner bucket: the discounted headhunting a new account gets, with its own guaranteed 6-star. 'Starter Headhunting' is the game's own name for it.",
    },
    "banner.status.active": {
        text: "Active",
        description: "Banner status pill: the banner is running right now.",
    },
    "banner.status.upcoming": {
        text: "Upcoming",
        description: "Banner status pill: the banner has been announced but has not opened yet.",
    },
    "banner.status.ended": {
        text: "Ended",
        description: "Banner status pill: the banner has closed.",
    },
    "banner.status.returning": {
        text: "Returning Doctors only",
        description: "Banner status pill: the banner is offered only to players coming back after a long absence, each on their own schedule, so it has no shared run window. 'Doctor' is how the game addresses the player.",
    },
    "banner.window.returning": {
        text: "No fixed dates, opens for each Doctor on their return",
        description: "Shown in place of a banner's date range when the game times it per player. 'Doctor' is how the game addresses the player.",
    },
    "banner.rule.normal": {
        text: "Standard headhunting",
        description: "Readable name for a banner's rule type: the ordinary rotating pool.",
    },
    "banner.rule.single": {
        text: "Debut rate-up",
        description: "Readable name for a banner's rule type: one newly released operator at a boosted rate. 'Rate-up' is the community term for a boosted drop chance.",
    },
    "banner.rule.limited": {
        text: "Limited",
        description: "Readable name for a banner's rule type: a limited-time pool.",
    },
    "banner.rule.linkage": {
        text: "Collab / joint operation",
        description: "Readable name for a banner's rule type: a crossover banner. 'Joint operation' is the game's own wording for it.",
    },
    "banner.rule.classic": {
        text: "Kernel headhunting",
        description: "Readable name for a banner's rule type: the pool of older operators. 'Kernel' is the game's own name for it.",
    },
    "banner.rule.classicAttain": {
        text: "Kernel · attain rate-up",
        description: "Readable name for a banner's rule type: a Kernel pool with a boosted rate on one operator. Keep the middle dot.",
    },
    "banner.rule.classicDouble": {
        text: "Kernel · double rate-up",
        description: "Readable name for a banner's rule type: a Kernel pool with two operators boosted. Keep the middle dot.",
    },
    "banner.rule.fesclassic": {
        text: "Kernel · anniversary",
        description: "Readable name for a banner's rule type: the anniversary Kernel pool. Keep the middle dot.",
    },
    "banner.rule.attain": {
        text: "Attain rate-up",
        description: "Readable name for a banner's rule type: one operator boosted and guaranteed within a pull count.",
    },
    "banner.rule.double": {
        text: "Double rate-up",
        description: "Readable name for a banner's rule type: two operators boosted at once.",
    },
    "banner.rule.backflow": {
        text: "Returning Headhunting",
        description: "Readable name for a banner's rule type: the recruitment banner offered only to players coming back after a long absence. The game's own English name.",
    },
    "banner.rule.special": {
        text: "Special",
        description: "Readable name for a banner's rule type: anything that fits none of the other buckets.",
    },
} satisfies MessageMap;

// `dynamic`: these keys are stored in the lookup tables in `constants.ts` and
// resolved by the consuming component as `t(BANNER_GROUP_LABEL_KEYS[group])`,
// so the extractor has no literal call site to match them against.
export const { keys } = defineMessages({ namespace, messages, dynamic: true });
