import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tools";

export const messages = {
    "squad.analysis.readiness.title": { text: "Readiness", description: "Title of the Squad Builder panel comparing how built each operator is." },
    "squad.analysis.readiness.question": { text: "Is it built evenly?", description: "One-line question under the Readiness panel's title." },
    "squad.analysis.readiness.usual": { text: "Usual promotion", description: "Stat cell label: the promotion most of the squad is at." },
    "squad.analysis.readiness.elite": { text: "Elite {n}", description: "A promotion level, e.g. Elite 2. Elite is the game's term." },
    "squad.analysis.readiness.usual.caption": { text: "{count} of {seats}", description: "Caption under the usual promotion: how many seats are at it out of the squad." },
    "squad.analysis.readiness.levels": { text: "Elite {n} levels", description: "Stat cell label: the typical level among seats at that promotion." },
    "squad.analysis.readiness.levels.caption": { text: "middle of the squad", description: "Caption under the typical level (it is the median)." },
    "squad.analysis.readiness.mastery": { text: "At mastery 3", description: "Stat cell label: how many chosen skills are at mastery 3." },
    "squad.analysis.readiness.mastery.caption": { text: "of {count} chosen skills", description: "Caption under the mastery 3 count." },
    "squad.analysis.readiness.modules": { text: "Modules on", description: "Stat cell label: how many seats have a module equipped." },
    "squad.analysis.readiness.modules.caption": { text: "of {count} that have one", description: "Caption under the modules count: out of the seats whose operator has a module." },
    "squad.analysis.readiness.level": { text: "Lv {level}", description: "An operator's level beside the level ring." },
    "squad.analysis.readiness.levelOf": { text: "Lv {level} of {cap}", description: "Hover text of the level ring: the level against that promotion's cap." },
    "squad.analysis.readiness.skill": { text: "Skill {n}, level {level}", description: "Hover text of the skill icon when the skill has no mastery." },
    "squad.analysis.readiness.skillMastery": { text: "Skill {n}, mastery {mastery}", description: "Hover text of the skill icon when the skill is mastered." },
    "squad.analysis.readiness.potential": { text: "Potential {n}", description: "Hover text of the potential icon." },
    "squad.analysis.readiness.moduleRank": { text: "R{level}", description: "Short module rank, e.g. R3." },
    "squad.analysis.readiness.noModule": { text: "No module", description: "Shown when the seat has no module equipped." },
    "squad.analysis.readiness.flag.level": { text: "Lv {value}. The other Elite {phase} seats sit around Lv {reference}.", description: "Flag on a seat whose level is far from the rest at its promotion." },
    "squad.analysis.readiness.flag.promotion": { text: "Elite {value}. The squad is mostly Elite {reference}.", description: "Flag on a seat at a different promotion than most of the squad." },
    "squad.analysis.readiness.flag.skillLocked": { text: "Skill {n} unlocks at Elite {value} Lv {reference}.", description: "Flag: the chosen skill is not unlocked at this build. {value} and {reference} are the promotion and level it unlocks at, from the game data." },
    "squad.analysis.readiness.flag.skillRankLocked": { text: "This skill rank needs Elite {value} Lv {reference}.", description: "Flag: the chosen skill level or mastery needs a higher promotion or level. {value} and {reference} are the promotion and level, from the game data." },
    "squad.analysis.readiness.flag.moduleLocked": { text: "The module unlocks at Elite {value} Lv {reference}.", description: "Flag: the chosen module is not unlocked at this build." },
    "squad.analysis.readiness.flag.supportLeftOut": { text: "Borrowed at the lender's build. Left out of the comparison.", description: "Note on the support operator's row." },
    "squad.analysis.readiness.footer": { text: "Hover a row to find its seat. The ring is the level against that promotion's cap.", description: "Footer of the Readiness panel." },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
