import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

/**
 * The registry modules are plain `.ts` - no React, so no `useT()`. Their
 * entries therefore carry message KEYS (`labelKey`, `descKey`) and the
 * consuming component resolves them with `t()`. The English lives here, in
 * the `nav` namespace the header and the command palette already use.
 */
export const namespace = "nav";

export const messages = {
    "toolCategory.calculator": {
        text: "Calculators",
        description: "Section heading over the number-crunching tools in the Tools menu and the tools index.",
    },
    "toolCategory.fun": {
        text: "For Fun",
        description: "Section heading over the non-serious tools (randomizer, birthdays) in the Tools menu and the tools index.",
    },
    "tool.recruitment.label": {
        text: "Recruitment calculator",
        description: "Name of the tool that works out which operators a set of recruitment tags can guarantee. 'Recruitment' is the in-game screen's name.",
    },
    "tool.recruitment.desc": {
        text: "Guaranteed tag combos · 1h parity",
        description: "One-line blurb under the recruitment calculator. '1h' is the one-hour recruitment duration; the separator is a middle dot.",
    },
    "tool.planner.label": {
        text: "Operator planner",
        description: "Name of the tool for planning what to invest in an operator.",
    },
    "tool.planner.desc": {
        text: "Plan promotions, skills, and modules",
        description: "One-line blurb under the operator planner. Promotion, skill and module are in-game progression systems.",
    },
    "tool.dps.label": {
        text: "DPS charts",
        description: "Name of the damage-per-second chart tool. 'DPS' is the usual abbreviation and stays as-is.",
    },
    "tool.dps.desc": {
        text: "Interactive damage curves per skill",
        description: "One-line blurb under the DPS charts tool.",
    },
    "tool.hps.label": {
        text: "HPS charts",
        description: "Name of the healing-per-second chart tool. 'HPS' is the usual abbreviation and stays as-is.",
    },
    "tool.hps.desc": {
        text: "Interactive healing curves per skill",
        description: "One-line blurb under the HPS charts tool.",
    },
    "tool.randomizer.label": {
        text: "Randomizer",
        description: "Name of the tool that picks a random squad.",
    },
    "tool.randomizer.desc": {
        text: "Pick a squad, break the meta",
        description: "Playful one-line blurb under the randomizer. 'Meta' is community jargon for the prevailing best picks.",
    },
    "tool.birthdays.label": {
        text: "Birthdays",
        description: "Name of the operator-birthday calendar tool.",
    },
    "tool.birthdays.desc": {
        text: "View and track operator birthdays",
        description: "One-line blurb under the birthdays tool.",
    },
    "tool.release.label": {
        text: "Release Planner",
        description: "Name of the tool that projects when Chinese-server content reaches the global servers.",
    },
    "tool.release.desc": {
        text: "When CN events, skins, and banners land on EN",
        description: "One-line blurb under the release planner. 'CN' and 'EN' are the Chinese and English game servers and stay as-is.",
    },
} satisfies MessageMap;

// `dynamic`: these keys are stored on a registry/constants entry and resolved
// by the consuming component as `t(item.labelKey)`, so the extractor has no
// literal call site to match them against.
export const { keys } = defineMessages({ namespace, messages, dynamic: true });
