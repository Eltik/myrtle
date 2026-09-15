import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tools";

export const messages = {
    "hps.breadcrumb.tools": {
        text: "Tools",
        description: "First crumb of the breadcrumb trail, naming the section this tool lives in.",
    },
    "hps.title": {
        text: "HPS Calculator",
        description: "Page heading and last breadcrumb. 'HPS' is healing per second, the usual abbreviation, and stays as-is.",
    },
    "hps.help.open": {
        text: "How does this work?",
        description: "Accessible name of the question-mark button that opens the explanation popover.",
    },
    "hps.help.title": {
        text: "How this works",
        description: "Heading inside the explanation popover.",
    },
    "hps.help.skillHps.term": {
        text: "Skill HPS",
        description: "Bold term opening a definition line in the explanation popover.",
    },
    "hps.help.skillHps.desc": {
        text: "is healing per second while the skill is active.",
        description: "Completes the sentence begun by the bold term 'Skill HPS', which is rendered immediately before it.",
    },
    "hps.help.baseHps.term": {
        text: "Base HPS",
        description: "Bold term opening a definition line in the explanation popover.",
    },
    "hps.help.baseHps.desc": {
        text: "is always-on healing during SP recharge - it's 0 for burst healers that only heal on activation.",
        description: "Completes the sentence begun by the bold term 'Base HPS'. 'SP' is the game's skill-point resource; the dash is a plain hyphen.",
    },
    "hps.help.averageHps.term": {
        text: "Average HPS",
        description: "Bold term opening a definition line in the explanation popover.",
    },
    "hps.help.averageHps.desc": {
        text: "averages skill uptime against SP recharge - the long-run sustainable rate.",
        description: "Completes the sentence begun by the bold term 'Average HPS'. The dash is a plain hyphen.",
    },
    "hps.help.sweep": {
        text: "Healing ignores enemy DEF/RES, so the chart sweeps a team variable instead: target count, ATK buff, or ASPD. Single-target vs. AoE healers diverge most along the Targets axis.",
        description: "Last line of the explanation popover. 'DEF', 'RES', 'ATK' and 'ASPD' are the game's abbreviations; 'AoE' is area of effect; 'Targets' names the axis tab of the same name.",
    },
    "hps.intro": {
        text: "Compare healer output across target counts and team buffs. Add multiple instances of the same operator to compare masteries, modules, and conditional setups side-by-side.",
        description: "Blurb under the page heading. 'Mastery' and 'module' are the game's own progression systems.",
    },
    "hps.note.label": {
        text: "Note:",
        description: "Bold lead-in of the attribution line under the page blurb. Keep the colon.",
    },
    "hps.note.creditLink": {
        text: "WhoAteMyCQQkie's",
        description: "Link text: the GitHub username of the author of the healing formulas, in the possessive. The name itself is never translated; only the possessive marking should adapt.",
    },
    "hps.note.credit": {
        text: "All calculations go to the credit of {link} GitHub repository.",
        description: "Attribution sentence under the page blurb. {link} is the author's GitHub username, labelled by hps.note.creditLink and linked to the repository, and may move wherever the sentence needs it.",
    },
    "hps.clearAll": {
        text: "Clear all",
        description: "Button that removes every configured healer, and the confirming button in its dialog.",
    },
    "hps.clearAll.title": {
        text: "Remove all healers?",
        description: "Title of the confirmation dialog for clearing the calculator.",
    },
    "hps.clearAll.desc": {
        text: "This removes all {count, plural, one {# configured healer} other {# configured healers}} from the calculator. Buff and chart settings stay.",
        description: "Body of the confirmation dialog for clearing the calculator.",
    },
    "hps.cancel": {
        text: "Cancel",
        description: "Button that dismisses the clear-all confirmation dialog without removing anything.",
    },
    "hps.healers": {
        text: "Healers",
        description: "Heading of the card holding the healer picker and the configured-healer count.",
    },
    "hps.healers.count": {
        text: "({count})",
        description: "Count beside the Healers heading when every configured healer is plotted. Keep the parentheses.",
    },
    "hps.healers.countVisible": {
        text: "({visible}/{total} visible)",
        description: "Count beside the Healers heading when some curves are hidden, e.g. '(2/4 visible)'.",
    },
    "hps.collapseAll": {
        text: "Collapse all",
        description: "Tooltip on the button that folds every healer card down to its header.",
    },
    "hps.expandAll": {
        text: "Expand all",
        description: "Tooltip on the button that opens every collapsed healer card.",
    },
    "hps.collapseAll.aria": {
        text: "Collapse all healer cards",
        description: "Accessible name of the collapse-all button; longer than the tooltip because it stands alone.",
    },
    "hps.expandAll.aria": {
        text: "Expand all healer cards",
        description: "Accessible name of the expand-all button; longer than the tooltip because it stands alone.",
    },
    "hps.download": {
        text: "Download chart as PNG",
        description: "Accessible name and tooltip of the button that saves the chart as an image.",
    },
    "hps.empty.title": {
        text: "No healers yet",
        description: "Empty-state heading in place of the chart before anything has been added.",
    },
    "hps.empty.desc": {
        text: "Pick any healer from the picker to plot an HPS curve. The chart will show how healing scales as you sweep across {axis}.",
        description: "Empty-state body in place of the chart. {axis} is the name of the current X axis, rendered in bold, and may move wherever the sentence needs it.",
    },
    "hps.configured": {
        text: "Configured healers",
        description: "Accessible name of the region holding the per-healer build cards.",
    },
    "hps.baseHps.burstHint": {
        text: "Burst healer - heals only on skill activation, so there's no off-skill (base) healing.",
        description: "Tooltip on a Base HPS figure of zero, explaining why. The dash is a plain hyphen.",
    },
    "hps.tooltip.targets": {
        text: "{count, plural, one {# target} other {# targets}}",
        description: "Chart tooltip header when sweeping the number of allies healed.",
    },
    "hps.tooltip.atk": {
        text: "ATK +{value}%",
        description: "Chart tooltip header when sweeping the attack buff. 'ATK' is the game's abbreviation.",
    },
    "hps.tooltip.aspd": {
        text: "ASPD +{value}",
        description: "Chart tooltip header when sweeping the attack-speed buff. 'ASPD' is the game's abbreviation.",
    },
    "hps.export.moduleSummary": {
        text: "{module} L{level}",
        description: "Module part of a chart-export legend row, e.g. 'SUM-X L3'. {module} is an in-game designator; 'L' abbreviates the module level.",
    },
    "hps.export.buildSummary": {
        text: "S{skill} · {module}",
        description: "Second line of a chart-export legend row: which skill, then the module. 'S1'/'S2'/'S3' is the game's own skill numbering.",
    },
    "hps.export.buffs": {
        text: "ATK +{atk}% · ASPD +{aspd} · {targets, plural, one {# target} other {# targets}}",
        description: "Buff setup printed in the chart-export title. 'ATK' and 'ASPD' are the game's abbreviations.",
    },
    "hps.export.title": {
        text: "{metric} vs {axis} - {buffs}",
        description: "Title printed above the exported chart image, e.g. 'Skill HPS vs Targets healed - ATK +0% · ASPD +0 · 1 target'. The dash is a plain hyphen.",
    },
    "hps.export.snapshotTargets": {
        text: "@ {count, plural, one {# target} other {# targets}}",
        description: "Heading of the value callout in the exported image, naming the target count the figures are read at. '@' means 'at'.",
    },
    "hps.export.snapshotAtk": {
        text: "@ ATK +{value}%",
        description: "Heading of the value callout in the exported image, naming the attack buff the figures are read at.",
    },
    "hps.export.snapshotAspd": {
        text: "@ ASPD +{value}",
        description: "Heading of the value callout in the exported image, naming the attack-speed buff the figures are read at.",
    },
    "hps.export.failed": {
        text: "Couldn't export chart: {error}",
        description: "Browser alert when saving the chart image failed. {error} is the underlying message and is not translated.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
