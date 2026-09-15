import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "operators";

export const messages = {
    "levelup.title": {
        text: "Level Up",
        description: "Heading of the Level-Up Cost tab.",
    },
    "levelup.subtitle": {
        text: "Materials needed to fully promote, master skills, and upgrade modules.",
        description: "Blurb under the Level-Up Cost tab heading.",
    },
    "levelup.empty.title": {
        text: "No upgrade costs",
        description: "Empty-state heading for an operator that needs no materials at all.",
    },
    "levelup.empty.body": {
        text: "{name} doesn''t require materials to level up.",
        description: "Empty-state body. {name} is the operator's name from the game data. The doubled apostrophe is ICU escaping and renders as one.",
    },
    "levelup.itemCount": {
        text: "×{count}",
        description: "How many of a material are needed, e.g. '×12'. The leading multiplication sign is the game's own notation; {count} may already be abbreviated.",
    },
    "levelup.countThousands": {
        text: "{value}k",
        description: "A material count of ten thousand or more, shortened to thousands, e.g. '180k'. Tight badge; use this language's short form for 'thousand'.",
    },
    "levelup.skillFallback": {
        text: "Skill {index}",
        description: "Stand-in name for a skill the game data does not name. {index} counts from 1.",
    },
    "levelup.phase.e0": {
        text: "E0",
        description: "Promotion stage shorthand: not yet promoted. The game's own notation, which normally stays as-is.",
    },
    "levelup.phase.e1": {
        text: "E1",
        description: "Promotion stage shorthand: Elite 1. The game's own notation, which normally stays as-is.",
    },
    "levelup.phase.e2": {
        text: "E2",
        description: "Promotion stage shorthand: Elite 2. The game's own notation, which normally stays as-is.",
    },
    "levelup.section.operatorLevel": {
        text: "Operator Level",
        description: "Section heading: the cost of levelling, as opposed to promoting.",
    },
    "levelup.section.operatorLevel.desc": {
        text: "Cost of leveling the operator to max level",
        description: "Subtitle under the Operator Level heading.",
    },
    "levelup.section.elite": {
        text: "Elite Promotion",
        description: "Section heading: the cost of each promotion stage. 'Elite' is the game's promotion tier.",
    },
    "levelup.section.elite.desc": {
        text: "Materials required to advance promotion stages",
        description: "Subtitle under the Elite Promotion heading.",
    },
    "levelup.section.skillLevels": {
        text: "Skill Levels",
        description: "Section heading: the cost of raising all skills from level 1 to 7 together.",
    },
    "levelup.section.skillLevels.desc": {
        text: "Cost to raise all skill levels (Rank 1 → 7)",
        description: "Subtitle under the Skill Levels heading. Keep the arrow.",
    },
    "levelup.section.mastery": {
        text: "Skill Mastery",
        description: "Section heading: the cost of the per-skill mastery ranks. 'Mastery' is the game's name for ranks M1 to M3.",
    },
    "levelup.section.mastery.desc": {
        text: "Materials needed for Mastery 1, 2, and 3",
        description: "Subtitle under the Skill Mastery heading.",
    },
    "levelup.section.modules": {
        text: "Modules",
        description: "Section heading: the cost of upgrading equipment modules. 'Module' is the game's name for the equipment system.",
    },
    "levelup.section.modules.desc": {
        text: "Upgrade materials for each equipment module",
        description: "Subtitle under the Modules heading.",
    },
    "levelup.section.total": {
        text: "Grand Total",
        description: "Section heading: every material added together.",
    },
    "levelup.section.total.desc": {
        text: "Every material needed to fully max this operator",
        description: "Subtitle under the Grand Total heading.",
    },
    "levelup.row.elite": {
        text: "Elite {phase}",
        description: "Row label naming a promotion stage, e.g. 'Elite 2'. {phase} is 0, 1 or 2.",
    },
    "levelup.row.levelRange": {
        text: "Level 1 to {max}",
        description: "Row hint naming the levels a cost covers, e.g. 'Level 1 to 80'.",
    },
    "levelup.row.promoteTo": {
        text: "Promote to E{to}",
        description: "Row hint for a promotion cost, e.g. 'Promote to E2'. 'E2' is the game's shorthand for Elite 2.",
    },
    "levelup.row.unlockAt": {
        text: "{phase} · Lv. {level}",
        description: "Row hint naming when an upgrade unlocks, e.g. 'E2 · Lv. 60'. {phase} is the promotion shorthand; keep the middle dot and abbreviate 'level' as the game does.",
    },
    "levelup.row.skillStep": {
        text: "Lv. {from} → {to}",
        description: "Row label for one skill-level step, e.g. 'Lv. 3 → 4'. Keep the arrow and abbreviate 'level' as the game does.",
    },
    "levelup.row.mastery": {
        text: "Mastery {index}",
        description: "Row label for one mastery rank, e.g. 'Mastery 2'.",
    },
    "levelup.row.stage": {
        text: "Stage {stage}",
        description: "Row label for one module upgrade stage, e.g. 'Stage 2'.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
