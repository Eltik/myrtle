import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "operators";

export const messages = {
    "cnUpdates.section.badge": {
        text: "CN",
        description: "Badge on the section that shows Chinese-server data. The server code, which normally stays as-is.",
    },
    "cnUpdates.section.title": {
        text: "Updates on CN",
        description: "Title of the operator-page section listing what the Chinese server's data has for this operator that this server's does not yet.",
    },
    "cnUpdates.section.desc": {
        text: "The CN server runs months ahead. This is its data where it differs from this server's: {changes}. Names and descriptions are CN's own text.",
        description: "Lead paragraph of the CN updates section. {changes} is a list such as 'new module and stats'. 'CN's own text' means untranslated Chinese.",
    },
    "cnUpdates.section.loading": {
        text: "Loading CN data...",
        description: "Shown while the Chinese server's record for this operator loads.",
    },
    "cnUpdates.section.unavailable": {
        text: "CN data for this operator could not be loaded.",
        description: "Shown when the Chinese server's record for this operator failed to load.",
    },
    "cnUpdates.section.newModule": {
        text: "New module",
        description: "Heading over a module the Chinese server has and this one does not yet.",
    },
    "cnUpdates.section.moduleStage": {
        text: "Stage {stage}",
        description: "Column heading for one upgrade stage (1 to 3) of a module.",
    },
    "cnUpdates.section.moduleTrait": {
        text: "Trait",
        description: "Label over the trait text a module stage gives, in CN's own text.",
    },
    "cnUpdates.section.moduleTalent": {
        text: "Talent",
        description: "Label over the talent text a module stage gives, in CN's own text.",
    },
    "cnUpdates.section.noBonuses": {
        text: "No stat bonuses",
        description: "Shown in a module stage column that adds no stats.",
    },
    "cnUpdates.section.changedModule": {
        text: "Module values",
        description: "Heading over a module whose numbers differ on CN.",
    },
    "cnUpdates.section.newSkill": {
        text: "New skill",
        description: "Heading over a skill the Chinese server lists and this one does not.",
    },
    "cnUpdates.section.skillValues": {
        text: "Skill values",
        description: "Heading over a skill whose numbers (SP cost, duration, multipliers) differ on CN.",
    },
    "cnUpdates.section.stats": {
        text: "Stats",
        description: "Heading over the base attributes that differ on CN.",
    },
    "cnUpdates.section.phase": {
        text: "Elite {phase}, level {level}",
        description: "Row group label for one promotion stage at its level cap, e.g. 'Elite 2, level 90'. 'Elite' is the game's word for a promotion.",
    },
    "cnUpdates.section.trust": {
        text: "Trust bonus",
        description: "Heading over the full-trust stat bonus when it differs on CN.",
    },
    "cnUpdates.section.talents": {
        text: "Talents",
        description: "Heading over talent values that differ on CN.",
    },
    "cnUpdates.section.trait": {
        text: "Trait",
        description: "Heading over trait values that differ on CN.",
    },
    "cnUpdates.section.range": {
        text: "The attack range differs on CN.",
        description: "One-line note when a promotion's attack range differs on the Chinese server.",
    },
    "cnUpdates.section.potentials": {
        text: "Potential bonuses differ on CN.",
        description: "One-line note when potential bonuses differ on the Chinese server.",
    },
    "cnUpdates.section.column.value": {
        text: "Value",
        description: "Column heading over the name of a number that differs, such as 'atk' or 'Redeploy'.",
    },
    "cnUpdates.section.column.here": {
        text: "Here",
        description: "Column heading over this server's number.",
    },
    "cnUpdates.section.column.cn": {
        text: "CN",
        description: "Column heading over the Chinese server's number. The server code, which normally stays as-is.",
    },
    "cnUpdates.section.absent": {
        text: "none",
        description: "Table cell when one server has no such value at all.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
