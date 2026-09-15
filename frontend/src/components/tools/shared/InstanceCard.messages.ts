import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

/**
 * The per-operator build card in the DPS and HPS calculators.
 *
 * Skill, module and potential names come from the game data and are rendered
 * as-is. The Elite (`E0`), skill (`S1`), module level (`L2`) and mastery
 * (`M3`) designators are the game's own and stay literal in the JSX.
 */
export const namespace = "tools";

export const messages = {
    "calc.instance.buildSummary": {
        text: "{skill} · {rank} · {module}",
        description: "One-line summary in the card header: skill name, skill rank (Lv 7 / M3), module. All three come from elsewhere; only the middle dots are ours.",
    },
    "calc.instance.hideCurve": {
        text: "Hide curve",
        description: "Accessible name and tooltip of the eye button, which removes this operator's line from the chart.",
    },
    "calc.instance.showCurve": {
        text: "Show curve",
        description: "Accessible name and tooltip of the eye button when the operator's line is currently hidden.",
    },
    "calc.instance.moveUp": {
        text: "Move up",
        description: "Accessible name and tooltip of the button that moves this card one place earlier in the list.",
    },
    "calc.instance.moveDown": {
        text: "Move down",
        description: "Accessible name and tooltip of the button that moves this card one place later in the list.",
    },
    "calc.instance.duplicate": {
        text: "Duplicate",
        description: "Accessible name and tooltip of the button that adds a second copy of this operator with the same build.",
    },
    "calc.instance.expand": {
        text: "Expand",
        description: "Accessible name and tooltip of the button that opens a collapsed card.",
    },
    "calc.instance.collapse": {
        text: "Collapse",
        description: "Accessible name and tooltip of the button that folds the card down to its header.",
    },
    "calc.instance.remove": {
        text: "Remove",
        description: "Accessible name and tooltip of the button that deletes this operator from the calculator.",
    },
    "calc.instance.promotion": {
        text: "Promotion",
        description: "Label over the row of Elite promotion buttons. 'Promotion' is the game's own progression step.",
    },
    "calc.instance.level": {
        text: "Level",
        description: "Label beside the slider that sets the operator's level.",
    },
    "calc.instance.potential": {
        text: "Potential",
        description: "Label over the row of potential-rank buttons. 'Potential' is the game's own progression system.",
    },
    "calc.instance.potentialAria": {
        text: "Potential {rank}",
        description: "Accessible name of one potential button; {rank} is 1-6.",
    },
    "calc.instance.skill": {
        text: "Skill",
        description: "Label over the row of buttons picking which of the operator's skills to simulate.",
    },
    "calc.instance.skillFallback": {
        text: "Skill {index}",
        description: "Stands in for a skill's name when the game data has none; {index} is the skill's 1-based slot.",
    },
    "calc.instance.useSkill": {
        text: "Use {skill}",
        description: "Accessible name of a skill button. {skill} is the skill's own name from the game data.",
    },
    "calc.instance.skillLevel": {
        text: "Skill level",
        description: "Label over the row of skill-level and mastery chips.",
    },
    "calc.instance.rankHint.e1": {
        text: "Promote to E1 for skill levels 5-7, E2 for masteries.",
        description: "Note under the skill-level chips at Elite 0. 'E1'/'E2' are the game's Elite promotions and 'masteries' its skill-mastery ranks.",
    },
    "calc.instance.rankHint.e2": {
        text: "Promote to E2 to unlock masteries.",
        description: "Note under the skill-level chips at Elite 1, where only masteries remain locked.",
    },
    "calc.instance.module": {
        text: "Module",
        description: "Label over the row of module buttons. 'Module' is the game's own equipment system.",
    },
    "calc.instance.moduleFallbackTooltip": {
        text: "Module {index}",
        description: "Tooltip on a module button when the game data carries no name for it; {index} is the engine's own module number.",
    },
    "calc.instance.moduleLevel": {
        text: "Module level",
        description: "Label over the row of module-level buttons (L1-L3).",
    },
    "calc.instance.trust": {
        text: "Trust",
        description: "Label beside the slider that sets trust. 'Trust' is the game's own affinity stat.",
    },
    "calc.instance.conditionals": {
        text: "Conditionals",
        description: "Label over the switches for an operator's conditional damage bonuses (trait, talent, skill and module riders that only apply in some situations).",
    },
    "calc.instance.externalBuffs": {
        text: "External buffs",
        description: "Toggle heading for the section holding buffs applied by OTHER operators rather than this one.",
    },
    "calc.instance.buff.atkPct": {
        text: "ATK %",
        description: "Input label for a percentage attack buff. 'ATK' is the game's own abbreviation and normally stays as-is.",
    },
    "calc.instance.buff.flatAtk": {
        text: "Flat ATK",
        description: "Input label for an attack buff given as a fixed number rather than a percentage.",
    },
    "calc.instance.buff.aspd": {
        text: "ASPD",
        description: "Input label for an attack-speed buff. The game's own abbreviation; normally unchanged.",
    },
    "calc.instance.buff.fragile": {
        text: "Fragile %",
        description: "Input label for a damage-amplification debuff on the enemy. 'Fragile' is the community's name for that effect.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
