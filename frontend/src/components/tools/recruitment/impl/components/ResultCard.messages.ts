import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tools";

export const messages = {
    "recruit.result.robotFloor": {
        text: "Robot",
        description: "Stands in for '1★' as the guaranteed floor, because the one-star tier is the game's Robot tier and a bare star would read as an ordinary 1★.",
    },
    "recruit.result.starFloor": {
        text: "{rarity}★",
        description: "The guaranteed floor as a star rating, e.g. '4★'.",
    },
    "recruit.result.guaranteed": {
        text: "Guaranteed {rarity}★",
        description: "Filled badge on a tag combination that cannot roll below five stars.",
    },
    "recruit.result.minTitle": {
        text: "Guaranteed minimum: {floor}",
        description: "Native tooltip on the quiet floor label. {floor} is either a star rating or the word Robot.",
    },
    "recruit.result.min": {
        text: "Min",
        description: "Label before the guaranteed floor on cards below a five-star lock. Abbreviation of 'minimum'; the space is very narrow.",
    },
    "recruit.result.noTags": {
        text: "No tags",
        description: "Shown in place of an operator's recruitment tags when it has none. Rendered in italics.",
    },
    "recruit.result.tags": {
        text: "Tags",
        description: "Heading over one operator's recruitment tags, in the hover card and in the expanded mobile row.",
    },
    "recruit.result.potentialAlt": {
        text: "Potential {rank}",
        description: "Alt text of the potential icon overlaid on an operator's portrait. {rank} is 1 to 6.",
    },
    "recruit.result.nextPotential": {
        text: "Next potential:",
        description: "Label in an operator's detail popover, before what the next potential rank grants (e.g. 'Next potential: ATK +28').",
    },
    "recruit.result.upgrade.unowned": {
        text: "Not owned",
        description: "Next-upgrade slot for an operator the signed-in roster does not hold: recruiting them is the gain.",
    },
    "recruit.result.upgrade.maxed": {
        text: "Maxed",
        description: "Next-upgrade slot for an operator already at the last potential rank.",
    },
    "recruit.result.upgrade.talent": {
        text: "Talent {n}",
        description: "Next potential improves the operator's n-th talent. Used only when the operator has more than one talent.",
    },
    "recruit.result.upgrade.talentOnly": {
        text: "Talent",
        description: "Next potential improves the operator's only talent.",
    },
    "recruit.result.upgrade.stat.COST": {
        text: "DP cost {value}",
        description: "Next potential changes deployment cost. {value} is signed, e.g. '-1'.",
    },
    "recruit.result.upgrade.stat.RESPAWN_TIME": {
        text: "Redeploy {value}s",
        description: "Next potential changes redeployment time in seconds. {value} is signed, e.g. '-4'.",
    },
    "recruit.result.upgrade.stat.ATK": {
        text: "ATK {value}",
        description: "Next potential changes attack. {value} is signed, e.g. '+28'.",
    },
    "recruit.result.upgrade.stat.DEF": {
        text: "DEF {value}",
        description: "Next potential changes defense. {value} is signed.",
    },
    "recruit.result.upgrade.stat.MAX_HP": {
        text: "HP {value}",
        description: "Next potential changes max HP. {value} is signed.",
    },
    "recruit.result.upgrade.stat.MAGIC_RESISTANCE": {
        text: "RES {value}",
        description: "Next potential changes arts resistance. {value} is signed.",
    },
    "recruit.result.upgrade.stat.ATTACK_SPEED": {
        text: "ASPD {value}",
        description: "Next potential changes attack speed. {value} is signed.",
    },
} satisfies MessageMap;

// `dynamic`: the stat labels are resolved as `t(`recruit.result.upgrade.stat.${attribute}`)`
// off the game's attribute type, so the extractor has no literal call site for them.
export const { keys } = defineMessages({ namespace, messages, dynamic: true });
