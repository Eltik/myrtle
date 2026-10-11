import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tools";

export const messages = {
    "squad.analysis.eyebrow": {
        text: "Squad analysis",
        description: "Small label above every analysis panel's title on the Squad Builder.",
    },
    "squad.analysis.empty": {
        text: "Put an operator in a seat to see this.",
        description: "Empty state of an analysis panel when the squad has no operators yet.",
    },
    "squad.analysis.unread": {
        text: "{names}: not on the EN server yet, so their talent, trait and skill wording is not read. Their talents show as not read, and rows that need the wording leave them out.",
        description: "Note above the analysis panels. The page reads talent and skill wording from the EN server's text by id, so an operator only out on this server has no reading yet. {names} is a comma-separated list of operator names.",
    },
    "squad.analysis.loading": {
        text: "Working it out",
        description: "Shown with a placeholder while the first analysis of the squad loads.",
    },
    "squad.analysis.error": {
        text: "{panel} could not be worked out. It tries again on your next change.",
        description: "Error line in an analysis panel. {panel} is the panel's title, e.g. Coverage or Economy.",
    },
    "squad.analysis.collapse": {
        text: "Hide {panel}",
        description: "Accessible label of the chevron that collapses an analysis panel. {panel} is the panel's title.",
    },
    "squad.analysis.expand": {
        text: "Show {panel}",
        description: "Accessible label of the chevron that expands a collapsed analysis panel. {panel} is the panel's title.",
    },
    "squad.analysis.chip.title": {
        text: "{name}: {why}",
        description: "Hover text of an operator chip in the squad analysis: the operator's name, then why the chip is there (e.g. 'Exusiai: Trait').",
    },
    "squad.analysis.chip.support": {
        text: "Support",
        description: "Small tag on the borrowed support operator's chip in the squad analysis.",
    },
    "squad.analysis.source.trait": {
        text: "Trait",
        description: "Where a fact about an operator comes from: the class trait.",
    },
    "squad.analysis.source.skill": {
        text: "S{n}, {name}",
        description: "Where a fact comes from: a skill. {n} is the skill number (1-3), {name} the skill's name.",
    },
    "squad.analysis.source.talent": {
        text: "Talent, {name}",
        description: "Where a fact comes from: a talent. {name} is the talent's name.",
    },
    "squad.analysis.source.module": {
        text: "Module, {name}",
        description: "Where a fact comes from: a module. {name} is the module's name.",
    },
    "squad.analysis.source.stats": {
        text: "Base stats",
        description: "Where a fact comes from: the operator's numbers, like block count or redeploy time.",
    },
    "squad.analysis.source.tag": {
        text: "Recruitment tag",
        description: "Where a fact comes from: one of the operator's recruitment tags.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
