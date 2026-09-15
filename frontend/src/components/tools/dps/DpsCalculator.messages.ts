import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tools";

export const messages = {
    "dps.breadcrumb.tools": {
        text: "Tools",
        description: "First crumb of the breadcrumb trail, naming the section this tool lives in.",
    },
    "dps.title": {
        text: "DPS Calculator",
        description: "Page heading and last breadcrumb. 'DPS' is damage per second, the usual abbreviation, and stays as-is.",
    },
    "dps.help.open": {
        text: "How does this work?",
        description: "Accessible name of the question-mark button that opens the explanation popover.",
    },
    "dps.help.title": {
        text: "How this works",
        description: "Heading inside the explanation popover.",
    },
    "dps.help.skillDps.term": {
        text: "Skill DPS",
        description: "Bold term opening a definition line in the explanation popover.",
    },
    "dps.help.skillDps.desc": {
        text: "is damage per second while the skill is active.",
        description: "Completes the sentence begun by the bold term 'Skill DPS', which is rendered immediately before it.",
    },
    "dps.help.averageDps.term": {
        text: "Average DPS",
        description: "Bold term opening a definition line in the explanation popover.",
    },
    "dps.help.averageDps.desc": {
        text: "averages skill uptime against SP recharge - the long-run sustainable rate.",
        description: "Completes the sentence begun by the bold term 'Average DPS'. 'SP' is the game's skill-point resource; the dash is a plain hyphen.",
    },
    "dps.help.totalDamage.term": {
        text: "Total Damage",
        description: "Bold term opening a definition line in the explanation popover.",
    },
    "dps.help.totalDamage.desc": {
        text: "is skill DPS × duration (or just skill DPS for passive skills).",
        description: "Completes the sentence begun by the bold term 'Total Damage'. Keep the multiplication sign.",
    },
    "dps.help.sweep": {
        text: "The chart sweeps the chosen X axis (DEF or RES). Whichever axis isn't being swept is held at the value you set in the Enemy panel.",
        description: "Last line of the explanation popover. 'DEF' and 'RES' are the game's abbreviations; 'Enemy panel' is the card of the same name on this page.",
    },
    "dps.intro": {
        text: "Compare operator output across enemy DEF or RES sweeps. Add multiple instances of the same operator to compare masteries, modules, and conditional setups side-by-side.",
        description: "Blurb under the page heading. 'Mastery' and 'module' are the game's own progression systems.",
    },
    "dps.note.label": {
        text: "Note:",
        description: "Bold lead-in of the attribution line under the page blurb. Keep the colon.",
    },
    "dps.note.creditLink": {
        text: "WhoAteMyCQQkie's",
        description: "Link text: the GitHub username of the author of the damage formulas, in the possessive. The name itself is never translated; only the possessive marking should adapt.",
    },
    "dps.note.credit": {
        text: "All calculations go to the credit of {link} GitHub repository.",
        description: "Attribution sentence under the page blurb. {link} is the author's GitHub username, labelled by dps.note.creditLink and linked to the repository, and may move wherever the sentence needs it.",
    },
    "dps.clearAll": {
        text: "Clear all",
        description: "Button that removes every configured operator, and the confirming button in its dialog.",
    },
    "dps.clearAll.title": {
        text: "Remove all operators?",
        description: "Title of the confirmation dialog for clearing the calculator.",
    },
    "dps.clearAll.desc": {
        text: "This removes all {count, plural, one {# configured operator} other {# configured operators}} from the calculator. Enemy and chart settings stay.",
        description: "Body of the confirmation dialog for clearing the calculator.",
    },
    "dps.cancel": {
        text: "Cancel",
        description: "Button that dismisses the clear-all confirmation dialog without removing anything.",
    },
    "dps.operators": {
        text: "Operators",
        description: "Heading of the card holding the operator picker and the configured-operator count.",
    },
    "dps.operators.count": {
        text: "({count})",
        description: "Count beside the Operators heading when every configured operator is plotted. Keep the parentheses.",
    },
    "dps.operators.countVisible": {
        text: "({visible}/{total} visible)",
        description: "Count beside the Operators heading when some curves are hidden, e.g. '(2/4 visible)'.",
    },
    "dps.collapseAll": {
        text: "Collapse all",
        description: "Tooltip on the button that folds every operator card down to its header.",
    },
    "dps.expandAll": {
        text: "Expand all",
        description: "Tooltip on the button that opens every collapsed operator card.",
    },
    "dps.collapseAll.aria": {
        text: "Collapse all operator cards",
        description: "Accessible name of the collapse-all button; longer than the tooltip because it stands alone.",
    },
    "dps.expandAll.aria": {
        text: "Expand all operator cards",
        description: "Accessible name of the expand-all button; longer than the tooltip because it stands alone.",
    },
    "dps.download": {
        text: "Download chart as PNG",
        description: "Accessible name and tooltip of the button that saves the chart as an image.",
    },
    "dps.empty.title": {
        text: "No operators yet",
        description: "Empty-state heading in place of the chart before anything has been added.",
    },
    "dps.empty.desc": {
        text: "Pick any operator from the picker to plot a DPS curve. The chart will show how DPS scales as you sweep across {axis}.",
        description: "Empty-state body in place of the chart. {axis} is the name of the current X axis, rendered in bold, and may move wherever the sentence needs it.",
    },
    "dps.configured": {
        text: "Configured operators",
        description: "Accessible name of the region holding the per-operator build cards.",
    },
    "dps.tooltip.def": {
        text: "DEF {value}",
        description: "Chart tooltip header when sweeping defense. 'DEF' is the game's abbreviation; {value} is already formatted.",
    },
    "dps.tooltip.res": {
        text: "RES {value}%",
        description: "Chart tooltip header when sweeping arts resistance. 'RES' is the game's abbreviation.",
    },
    "dps.export.moduleSummary": {
        text: "{module} L{level}",
        description: "Module part of a chart-export legend row, e.g. 'SUM-X L3'. {module} is an in-game designator; 'L' abbreviates the module level.",
    },
    "dps.export.buildSummary": {
        text: "S{skill} · {module}",
        description: "Second line of a chart-export legend row: which skill, then the module. 'S1'/'S2'/'S3' is the game's own skill numbering.",
    },
    "dps.export.enemy": {
        text: "DEF {defense} · RES {res}% · {targets, plural, one {# target} other {# targets}}",
        description: "Enemy setup printed in the chart-export title. 'DEF' and 'RES' are the game's abbreviations.",
    },
    "dps.export.title": {
        text: "{metric} vs {axis} - {enemy}",
        description: "Title printed above the exported chart image, e.g. 'Skill DPS vs Enemy DEF - DEF 0 · RES 0% · 1 target'. The dash is a plain hyphen.",
    },
    "dps.export.axis.def": {
        text: "Enemy DEF",
        description: "Axis name in the chart-export title when sweeping defense. The game's own abbreviation.",
    },
    "dps.export.axis.res": {
        text: "Enemy RES",
        description: "Axis name in the chart-export title when sweeping arts resistance. The game's own abbreviation.",
    },
    "dps.export.snapshotDef": {
        text: "@ DEF {defense}",
        description: "Heading of the value callout in the exported image, naming the defense the figures are read at. '@' means 'at'.",
    },
    "dps.export.snapshotRes": {
        text: "@ RES {res}%",
        description: "Heading of the value callout in the exported image, naming the arts resistance the figures are read at.",
    },
    "dps.export.failed": {
        text: "Couldn't export chart: {error}",
        description: "Browser alert when saving the chart image failed. {error} is the underlying message and is not translated.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
