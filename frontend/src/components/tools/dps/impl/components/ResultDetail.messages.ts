import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

/** Per-card result details in the DPS results panel and the ranking table's shared labels. */
export const namespace = "tools";

export const messages = {
    "dps.category.sustained": {
        text: "Sustained",
        description: "Skill shape: its damage is steady (no skill, passive, or permanent skill).",
    },
    "dps.category.burst": {
        text: "Burst",
        description: "Skill shape: a timed or ammo window that recharges.",
    },
    "dps.category.instant": {
        text: "Instant",
        description: "Skill shape: a single activation with no duration.",
    },
    "dps.detail.startup": {
        text: "First cast {seconds}s",
        description: "Seconds from deployment until the skill can first be used. Compact.",
    },
    "dps.detail.cycle": {
        text: "Cycle {seconds}s",
        description: "Seconds per full skill cycle: skill duration plus recharge. Compact.",
    },
    "dps.detail.split": {
        text: "Main {primary} · others {secondary}",
        description: "How the damage splits between the main target and the secondary targets, in the chosen metric.",
    },
    "dps.detail.spNotModelled": {
        text: "SP on hit not modelled",
        description: "Caveat badge: this skill charges SP when the operator is hit, which the cycle model does not simulate; the classic average is shown.",
    },
    "dps.detail.spNotModelled.hint": {
        text: "This skill charges when the operator takes hits. The SP-aware model cannot time that, so average DPS falls back to the classic one and no timings are shown.",
        description: "Tooltip of the 'SP on hit not modelled' caveat.",
    },
    "dps.detail.normalAttack": {
        text: "Plain attack off-skill",
        description: "Caveat badge: between skills, a generic basic attack was used because this operator's own formula has no basic-attack path.",
    },
    "dps.detail.normalAttack.hint": {
        text: "This operator's formula cannot compute its attack without the skill, so the time between skills uses a generic basic attack.",
        description: "Tooltip of the 'Plain attack off-skill' caveat.",
    },
    "dps.detail.approxSplit": {
        text: "Typed Fragile approximated",
        description: "Caveat badge: physical- or arts-only damage-taken buffs were applied approximately for this operator.",
    },
    "dps.detail.approxSplit.hint": {
        text: "This operator's damage reacts to both DEF and RES (or to neither), so only the generic damage-taken group and the one matching its attack type were applied.",
        description: "Tooltip of the 'Typed Fragile approximated' caveat.",
    },
    "dps.detail.clamped": {
        text: "Secondary share clamped",
        description: "Caveat badge: the computed damage on secondary targets came out negative and was set to 0.",
    },
    "dps.detail.clamped.hint": {
        text: "Hitting more targets lowered this operator's total, so the secondary share came out negative and is shown as 0. Main plus others no longer equals the multi-target total.",
        description: "Tooltip of the 'Secondary share clamped' caveat.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
