import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tools";

export const messages = {
    "dps.enemy.title": {
        text: "Enemy & target",
        description: "Card heading over the controls describing what the operators are shooting at.",
    },
    "dps.enemy.desc": {
        text: "DEF and RES set the KPI snapshot, and the axis you're not sweeping is held at this value.",
        description: "Caption under the 'Enemy & target' heading. 'DEF' and 'RES' are the game's abbreviations for defense and arts resistance; 'KPI snapshot' is the result panel beside the chart.",
    },
    "dps.enemy.presets": {
        text: "Quick presets",
        description: "Label over the row of one-click enemy setups.",
    },
    "dps.enemy.snapshotDef": {
        text: "Snapshot DEF",
        description: "Input label for the defense value the result panel is computed at.",
    },
    "dps.enemy.snapshotRes": {
        text: "Snapshot RES (%)",
        description: "Input label for the arts-resistance value the result panel is computed at.",
    },
    "dps.enemy.targets": {
        text: "Targets hit (AoE)",
        description: "Input label for how many enemies an attack hits at once. 'AoE' is area of effect, the usual abbreviation.",
    },
    "dps.enemy.spBoost": {
        text: "Bonus SP/s",
        description: "Input label for extra skill points per second from outside sources. 'SP' is the game's skill-point resource.",
    },
    "dps.enemy.shred": {
        text: "Shred",
        description: "Toggle heading for the section holding defense- and resistance-reduction values. 'Shred' is community jargon for lowering an enemy's DEF or RES.",
    },
    "dps.enemy.shred.on": {
        text: "on",
        description: "Tiny badge beside the 'Shred' heading when at least one shred value is non-zero. Lowercase; it sits in a very small pill.",
    },
    "dps.enemy.shred.defPct": {
        text: "DEF % shred",
        description: "Input label for defense reduction expressed as a percentage.",
    },
    "dps.enemy.shred.defFlat": {
        text: "DEF flat",
        description: "Input label for defense reduction expressed as a fixed number.",
    },
    "dps.enemy.shred.resPct": {
        text: "RES % shred",
        description: "Input label for arts-resistance reduction expressed as a percentage.",
    },
    "dps.enemy.shred.resFlat": {
        text: "RES flat",
        description: "Input label for arts-resistance reduction expressed as a fixed number.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
