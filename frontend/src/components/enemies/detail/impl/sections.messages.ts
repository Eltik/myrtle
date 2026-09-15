import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "enemies";

export const messages = {
    "overview.description": {
        text: "Description",
        description: "Kicker over the enemy's handbook blurb. The blurb itself is game data.",
    },
    "overview.traits": {
        text: "Traits",
        description: "Kicker over the enemy's ability list. The ability text itself is game data.",
    },
    "overview.tags": {
        text: "Tags",
        description: "Kicker over the enemy's tag pills. The tags themselves are game data.",
    },
    "overview.metadata": {
        text: "Metadata",
        description: "Kicker over the grid of raw backend fields for this enemy.",
    },
    "overview.meta.enemyId": {
        text: "Enemy ID",
        description: "Metadata field label; its value is a game-data identifier and is never translated.",
    },
    "overview.meta.sortId": {
        text: "Sort ID",
        description: "Metadata field label: the number the game orders the handbook by.",
    },
    "overview.meta.race": {
        text: "Race",
        description: "Metadata field label; its value is a race name from the game data.",
    },
    "overview.meta.handbook": {
        text: "In Handbook",
        description: "Metadata field label: whether the game shows this enemy in its own handbook screen.",
    },
    "overview.meta.hidden": {
        text: "Hidden",
        description: "Value of the In Handbook field when the game hides this enemy from its handbook.",
    },
    "overview.meta.visible": {
        text: "Visible",
        description: "Value of the In Handbook field when the game lists this enemy in its handbook.",
    },
    "stats.none": {
        text: "No stats available for this enemy.",
        description: "Shown in the Stats tab when the game data ships no stat block at all.",
    },
    "stats.phase.single": {
        text: "Stats",
        description: "Heading of the only stat block, when the enemy has just one. Sits in a narrow pill.",
    },
    "stats.phase.level": {
        text: "Level {level}",
        description: "Heading of one stat block when the enemy has several and the handbook names none of them. {level} counts from 0, as the game data does.",
    },
    "stats.combat": {
        text: "Combat",
        description: "Kicker over the stat blocks when the enemy has only one.",
    },
    "stats.combat.forms": {
        text: "Combat · {count, plural, one {# form} other {# forms}}",
        description: "Kicker over the stat blocks when the handbook names each of them, so they read as the enemy's transformation forms.",
    },
    "stats.combat.levels": {
        text: "Combat · {count, plural, one {# level} other {# levels}}",
        description: "Kicker over the stat blocks when the handbook names none of them, so they read as the game data's numbered levels.",
    },
    "stats.row.maxHp": {
        text: "Max HP",
        description: "Stat row label: maximum hit points.",
    },
    "stats.row.atk": {
        text: "ATK",
        description: "Stat row label: attack. Abbreviated to fit a narrow two-column list.",
    },
    "stats.row.def": {
        text: "DEF",
        description: "Stat row label: defense. Abbreviated to fit a narrow two-column list.",
    },
    "stats.row.res": {
        text: "RES %",
        description: "Stat row label: magic resistance as a percentage. Abbreviated to fit a narrow two-column list.",
    },
    "stats.row.moveSpeed": {
        text: "Move Speed",
        description: "Stat row label: tiles travelled per second.",
    },
    "stats.row.aspd": {
        text: "ASPD",
        description: "Stat row label: attack speed. The game community's abbreviation; keep it short.",
    },
    "stats.row.baseAttackTime": {
        text: "Base ATK Time",
        description: "Stat row label: seconds between attacks before attack-speed scaling.",
    },
    "stats.row.weight": {
        text: "Weight",
        description: "Stat row label: mass level, which decides how far the enemy can be shoved.",
    },
    "stats.immunities": {
        text: "Immunities",
        description: "Kicker over the table of status effects the enemy shrugs off.",
    },
    "stats.immunity.status": {
        text: "Status",
        description: "First column header of the immunity table, over the status-effect names.",
    },
    "stats.immunity.value": {
        text: "Value",
        description: "Only value column header of the immunity table, used when the enemy has a single stat block so there is no form to name.",
    },
    "stats.immunity.stun": {
        text: "Stun",
        description: "Status effect in the immunity table. The game's own term for it.",
    },
    "stats.immunity.silence": {
        text: "Silence",
        description: "Status effect in the immunity table: skills blocked. The game's own term for it.",
    },
    "stats.immunity.sleep": {
        text: "Sleep",
        description: "Status effect in the immunity table. The game's own term for it.",
    },
    "stats.immunity.frozen": {
        text: "Frozen",
        description: "Status effect in the immunity table. The game's own term for it.",
    },
    "stats.immunity.levitate": {
        text: "Levitate",
        description: "Status effect in the immunity table: lifted off the ground. The game's own term for it.",
    },
    "stats.immunity.aria.immune": {
        text: "{status} immune in {phase}",
        description: "Accessible name of a tick in the immunity table. {status} is a status effect and {phase} the stat block's heading.",
    },
    "stats.immunity.aria.vulnerable": {
        text: "{status} vulnerable in {phase}",
        description: "Accessible name of a cross in the immunity table. {status} is a status effect and {phase} the stat block's heading.",
    },
    "stats.forms.note": {
        text: "This enemy transforms across {forms} forms in-fight, but Hypergryph's data ships {count, plural, one {a single stat block} other {# stat blocks}}. Subsequent forms modify behavior through skill triggers rather than swapping the base HP/ATK/DEF - see {traits} for the form descriptions.",
        description: "Note explaining why an enemy with several narrative forms has fewer stat blocks. {forms} is the bold form count and {traits} the bold pointer labelled by stats.forms.note.traits; both may move wherever the sentence needs them. 'Hypergryph' is the game's developer; keep the name.",
    },
    "stats.forms.note.traits": {
        text: "Overview · Traits",
        description: "The bold pointer inside that note, naming this page's Overview tab and its Traits section. Must match those two labels.",
    },
    "skills.none": {
        text: "No skill data available for this enemy.",
        description: "Shown in the Skills tab when no stat block carries any skill.",
    },
    "skills.identical.forms": {
        text: "Skill kit is identical across all {phases} {count, plural, one {form} other {forms}}.",
        description: "Shown above the skill list when every stat block carries the same skills and the handbook names each one as a form. {phases} is the bold count and may move; {count} is the same number again, used only to pick the plural form. 'Kit' is the set of skills an enemy has.",
    },
    "skills.identical.levels": {
        text: "Skill kit is identical across all {phases} {count, plural, one {level} other {levels}}.",
        description: "The same line when the handbook names none of the stat blocks, so they are levels rather than forms. {phases} is the bold count and may move; {count} is the same number again, used only to pick the plural form.",
    },
    "skills.empty.form": {
        text: "No skills active in this form.",
        description: "Shown under one stat block's heading when that form has no skills, and the handbook names the forms.",
    },
    "skills.empty.level": {
        text: "No skills active in this level.",
        description: "Shown under one stat block's heading when that level has no skills, and the handbook names none of them.",
    },
    "skills.cooldown": {
        text: "Cooldown",
        description: "Skill card field label: seconds between uses.",
    },
    "skills.initCd": {
        text: "Init CD",
        description: "Skill card field label: the cooldown the skill starts the battle on. 'CD' abbreviates cooldown; very tight space.",
    },
    "skills.spCost": {
        text: "SP Cost",
        description: "Skill card field label: skill points needed to fire. 'SP' is the game's own abbreviation.",
    },
} satisfies MessageMap;

// `dynamic`: the stat-row and immunity-row keys live on module-level tables and
// are resolved as `t(row.labelKey)`, so the extractor has no literal call site.
export const { keys } = defineMessages({ namespace, messages, dynamic: true });
