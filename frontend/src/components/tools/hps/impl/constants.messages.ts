import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

/**
 * The axis, metric and buff-preset tables in `constants.ts` are plain data in
 * a module with no React, so they carry message KEYS and the component that
 * renders a row resolves it with `t()`.
 */
export const namespace = "tools";

export const messages = {
    "hps.metric.skillHps": {
        text: "Skill HPS",
        description: "Y-axis metric: healing per second while the skill is running. 'HPS' is the usual abbreviation and stays as-is.",
    },
    "hps.metric.baseHps": {
        text: "Base HPS",
        description: "Y-axis metric: healing per second with no skill running.",
    },
    "hps.metric.avgHps": {
        text: "Average HPS",
        description: "Y-axis metric: healing per second averaged over skill uptime and recharge.",
    },
    "hps.metric.skillHps.short": {
        text: "Skill HPS",
        description: "Column heading for skill HPS in the result panel. Very narrow column.",
    },
    "hps.metric.baseHps.short": {
        text: "Base HPS",
        description: "Column heading for base HPS in the result panel. Very narrow column.",
    },
    "hps.metric.avgHps.short": {
        text: "Avg HPS",
        description: "Column heading for average HPS in the result panel. Abbreviated because the column is very narrow.",
    },
    "hps.metric.skillHps.hint": {
        text: "Healing per second while the skill is active.",
        description: "Caption explaining the Skill HPS metric when it is selected.",
    },
    "hps.metric.baseHps.hint": {
        text: "Always-on healing per second during SP recharge (0 for burst healers).",
        description: "Caption explaining the Base HPS metric. 'SP' is the game's skill-point resource; a burst healer only heals when its skill fires.",
    },
    "hps.metric.avgHps.hint": {
        text: "Sustained HPS averaged over a full skill + recharge cycle.",
        description: "Caption explaining the Average HPS metric.",
    },
    "hps.axis.targets": {
        text: "Targets healed",
        description: "X-axis name: how many allies are being healed at once.",
    },
    "hps.axis.atk": {
        text: "ATK buff (%)",
        description: "X-axis name: the attack buff the healer is receiving. 'ATK' is the game's own abbreviation.",
    },
    "hps.axis.aspd": {
        text: "ASPD buff",
        description: "X-axis name: the attack-speed buff the healer is receiving. 'ASPD' is the game's own abbreviation.",
    },
    "hps.axis.targets.short": {
        text: "Targets",
        description: "Compact label for the target-count axis tab on narrow screens.",
    },
    "hps.axis.atk.short": {
        text: "ATK %",
        description: "Compact label for the attack-buff axis tab on narrow screens. The game's own abbreviation.",
    },
    "hps.axis.aspd.short": {
        text: "ASPD",
        description: "Compact label for the attack-speed axis tab on narrow screens. The game's own abbreviation.",
    },
    "hps.preset.solo": {
        text: "Solo",
        description: "Buff preset: the healer alone, with no help and one target.",
    },
    "hps.preset.solo.summary": {
        text: "No buffs · 1 target",
        description: "Tooltip listing what the 'Solo' preset applies.",
    },
    "hps.preset.atkAura": {
        text: "ATK aura",
        description: "Buff preset: a standing attack buff from another operator. 'ATK' is the game's abbreviation; 'aura' is community jargon for an always-on team buff.",
    },
    "hps.preset.atkAura.summary": {
        text: "ATK +40% · 1 target",
        description: "Tooltip listing what the 'ATK aura' preset applies.",
    },
    "hps.preset.aspd": {
        text: "ASPD push",
        description: "Buff preset: a heavy attack-speed buff. 'ASPD' is the game's abbreviation.",
    },
    "hps.preset.aspd.summary": {
        text: "ASPD +60 · 1 target",
        description: "Tooltip listing what the 'ASPD push' preset applies.",
    },
    "hps.preset.aoe": {
        text: "AoE",
        description: "Buff preset: healing several allies at once, unbuffed. 'AoE' is area of effect, the usual abbreviation.",
    },
    "hps.preset.aoe.summary": {
        text: "No buffs · 3 targets",
        description: "Tooltip listing what the 'AoE' preset applies.",
    },
    "hps.preset.buffedAoe": {
        text: "Buffed AoE",
        description: "Buff preset: healing several allies at once with an attack buff running.",
    },
    "hps.preset.buffedAoe.summary": {
        text: "ATK +40% · 3 targets",
        description: "Tooltip listing what the 'Buffed AoE' preset applies.",
    },
} satisfies MessageMap;

// `dynamic`: these keys are stored on a constants entry and resolved by the
// consuming component as `t(item.labelKey)`, so the extractor has no literal
// call site to match them against.
export const { keys } = defineMessages({ namespace, messages, dynamic: true });
