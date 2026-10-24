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
    "dps.enemy.secondary": {
        text: "Secondary targets",
        description: "Switch label: add more enemies beside the main one, with their own DEF and RES. The result splits damage between the main target and the rest.",
    },
    "dps.enemy.secondary.desc": {
        text: "Other enemies hit alongside the main one. Results split into main and secondary damage.",
        description: "One-line explanation under the secondary-targets switch.",
    },
    "dps.enemy.secondary.def": {
        text: "Their DEF",
        description: "Input label: the secondary enemies' defense. 'DEF' is the game's abbreviation.",
    },
    "dps.enemy.secondary.res": {
        text: "Their RES",
        description: "Input label: the secondary enemies' arts resistance. 'RES' is the game's abbreviation.",
    },
    "dps.enemy.secondary.count": {
        text: "How many",
        description: "Input label: the number of secondary enemies.",
    },
    "dps.enemy.secondary.targetsNote": {
        text: "With secondary targets on, the main target is one enemy and the count above sets the rest.",
        description: "Note shown when secondary targets replace the plain target count.",
    },
    "dps.enemy.cycle": {
        text: "Skill cycle model",
        description: "Label of the selector for how average DPS over a skill cycle is computed.",
    },
    "dps.enemy.cycle.legacy": {
        text: "Classic",
        description: "Cycle model option: the original average, a plain basic attack between skills.",
    },
    "dps.enemy.cycle.v2": {
        text: "SP-aware",
        description: "Cycle model option: the operator's own attack between skills and SP charged by the skill's SP type.",
    },
    "dps.enemy.cycle.desc.legacy": {
        text: "Average DPS uses a plain basic attack between skills and charges every skill by time.",
        description: "One-line explanation of the classic cycle model.",
    },
    "dps.enemy.cycle.desc.v2": {
        text: "Average DPS uses the operator's own attack between skills and charges SP by the skill's type (per second or per hit); it also reports start-up and cycle times.",
        description: "One-line explanation of the SP-aware cycle model.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
