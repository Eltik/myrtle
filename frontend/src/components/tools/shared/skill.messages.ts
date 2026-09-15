import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

/**
 * Labels the skill-rank model in `skill.ts` derives. That module has no React,
 * so the label helpers take an optional `t` and fall back to the bundled
 * source catalog; the text itself lives here.
 */
export const namespace = "tools";

export const messages = {
    "calc.skillRank.level": {
        text: "Lv {rank}",
        description: "Tooltip for a pre-mastery skill level chip. 'Lv' abbreviates 'level'; {rank} is 1-7.",
    },
    "calc.skillRank.mastery": {
        text: "M{mastery}",
        description: "Tooltip for a mastery chip. 'M1'/'M2'/'M3' is the game's own name for the three mastery ranks, so it normally stays as-is.",
    },
    "calc.skillRank.levelShort": {
        text: "L{rank}",
        description: "Very compact skill-level label used in the one-line build summary, e.g. 'L7'.",
    },
    "calc.skillRank.masteryShort": {
        text: "M{mastery}",
        description: "Very compact mastery label used in the one-line build summary. The game's own name; normally unchanged.",
    },
    "calc.skillRank.lock.mastery": {
        text: "Masteries require E2",
        description: "Tooltip on a disabled mastery chip. 'E2' is the game's second Elite promotion.",
    },
    "calc.skillRank.lock.level": {
        text: "Skill levels above 4 require E1",
        description: "Tooltip on a disabled skill-level chip. 'E1' is the game's first Elite promotion.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
