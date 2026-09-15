import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tools";

export const messages = {
    "hps.buffs.title": {
        text: "Team buffs & targets",
        description: "Card heading over the controls describing the help the healer is getting and how many allies it heals.",
    },
    "hps.buffs.desc": {
        text: "Healing ignores enemy DEF/RES, so these team-side buffs drive output. The axis you're sweeping is held at this value for the snapshot.",
        description: "Caption under the 'Team buffs & targets' heading. 'DEF' and 'RES' are the game's abbreviations for defense and arts resistance.",
    },
    "hps.buffs.presets": {
        text: "Quick presets",
        description: "Label over the row of one-click buff setups.",
    },
    "hps.buffs.atkPct": {
        text: "ATK %",
        description: "Input label for a percentage attack buff. 'ATK' is the game's own abbreviation.",
    },
    "hps.buffs.flatAtk": {
        text: "Flat ATK",
        description: "Input label for an attack buff given as a fixed number rather than a percentage.",
    },
    "hps.buffs.aspd": {
        text: "ASPD",
        description: "Input label for an attack-speed buff. The game's own abbreviation.",
    },
    "hps.buffs.healAmp": {
        text: "Heal amp %",
        description: "Input label for a healing-amplification buff. 'amp' abbreviates 'amplification'; the field is narrow.",
    },
    "hps.buffs.targets": {
        text: "Targets healed",
        description: "Input label for how many allies are healed at once.",
    },
    "hps.buffs.spBoost": {
        text: "Bonus SP/s",
        description: "Input label for extra skill points per second from outside sources. 'SP' is the game's skill-point resource.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
