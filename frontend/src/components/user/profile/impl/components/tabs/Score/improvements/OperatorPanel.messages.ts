import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

/**
 * The Operator panel's upgrade tags and score axes live in lookup tables keyed
 * by the API's tag / dimension codes, so those entries hold message KEYS and
 * the panel resolves them with `t()`.
 *
 * `Elite`, `Mastery`, `Module`, `Potential` and `Trust` are the game's own
 * progression systems - the game ships its own wording per region, so a
 * translation should follow it.
 */
export const namespace = "user";

export const messages = {
    "score.improvements.operator.empty": {
        text: "Every owned operator is at their last milestone - nothing to upgrade here.",
        description: "Empty state in the Operator panel when nothing is left to invest in.",
    },
    "score.improvements.operator.title": {
        text: "Operators below milestone",
        description: "Heading over the list of operators that still have an upgrade available.",
    },
    "score.improvements.operator.count": {
        text: "{n} total · +{gain} to overall grade",
        description: "Count badge beside that heading. {n} is how many operators are listed, {gain} the points finishing all of them would add to the overall grade. Keep the middle dot.",
    },
    "score.improvements.operator.byType": {
        text: "By upgrade type · % gained in this section",
        description: "Kicker over the grid of upgrade-type tiles. Keep the middle dot.",
    },
    "score.improvements.operator.clearFilter": {
        text: "Clear filter ({n} shown)",
        description: "Button that drops the active upgrade-type filter. {n} is how many operators the filter leaves visible.",
    },
    "score.improvements.operator.noMatch": {
        text: "No operators match this filter.",
        description: "Empty state under the upgrade-type tiles when the selected filter leaves nothing.",
    },
    "score.improvements.operator.footnote.headerGain": {
        text: "+{gain} to overall grade",
        description: "The highlighted figure quoted back from the heading inside the footnote. Same wording as the count badge's second half.",
    },
    "score.improvements.operator.footnote.sectionUnit": {
        text: "% of this section",
        description: "The highlighted unit inside the footnote: the figures are percentages of the Operator section, not of the overall grade.",
    },
    "score.improvements.operator.footnote": {
        text: "The header's {gain} is what finishing every upgrade here would add to your headline score. The per-tag, per-rarity, and per-operator figures below are {unit} (the percentage shown at the top of the card) - there's +{room}% of room left here to climb toward 100%. ELITE includes a full re-level at the new phase, so it overlaps MAX_LEVEL; the per-op total deduplicates the pair.",
        description:
            "Footnote under the operator list, explaining the units. {gain} is the highlighted figure quoted back from the heading (score.improvements.operator.footnote.headerGain) and {unit} the highlighted unit (score.improvements.operator.footnote.sectionUnit); both may move. ELITE and MAX_LEVEL are raw API tag codes and stay as-is; 'per-op' is short for per-operator.",
    },
    "score.improvements.operator.tag.elite.label": {
        text: "E↑",
        description: "Six-character tile label for the Elite-promotion upgrade. 'E' is the game's abbreviation for Elite; the arrow means 'raise'.",
    },
    "score.improvements.operator.tag.elite.desc": {
        text: "Elite promotion",
        description: "Name of the Elite-promotion upgrade, under its tile and opening its tooltip.",
    },
    "score.improvements.operator.tag.elite.detail": {
        text: "promoting to max elite and re-leveling to its new cap",
        description: "Mid-sentence continuation of the Elite tooltip, after 'Elite promotion - '. Lowercase on purpose.",
    },
    "score.improvements.operator.tag.maxLevel.label": {
        text: "Lvl",
        description: "Six-character tile label for the level-cap upgrade. 'Lvl' is the game's abbreviation for level.",
    },
    "score.improvements.operator.tag.maxLevel.desc": {
        text: "Level cap",
        description: "Name of the level-cap upgrade, under its tile and opening its tooltip.",
    },
    "score.improvements.operator.tag.maxLevel.detail": {
        text: "leveling to the cap of this operator's current elite phase",
        description: "Mid-sentence continuation of the level-cap tooltip. Lowercase on purpose.",
    },
    "score.improvements.operator.tag.m3.label": {
        text: "M3",
        description: "Six-character tile label for Mastery 3. The game's own abbreviation, normally unchanged.",
    },
    "score.improvements.operator.tag.m3.desc": {
        text: "Mastery 3",
        description: "Name of the Mastery-3 upgrade. 'Mastery' is the game's skill-rank system.",
    },
    "score.improvements.operator.tag.m3.detail": {
        text: "raising one skill to Mastery 3",
        description: "Mid-sentence continuation of the Mastery-3 tooltip. Lowercase on purpose.",
    },
    "score.improvements.operator.tag.sl7.label": {
        text: "SL7",
        description: "Six-character tile label for skill level 7. The game's own abbreviation, normally unchanged.",
    },
    "score.improvements.operator.tag.sl7.desc": {
        text: "Skill 7",
        description: "Name of the skill-level-7 upgrade.",
    },
    "score.improvements.operator.tag.sl7.detail": {
        text: "raising the shared skill level to 7",
        description: "Mid-sentence continuation of the skill-level tooltip. The skill level is shared across an operator's skills. Lowercase on purpose.",
    },
    "score.improvements.operator.tag.mod3.label": {
        text: "Mod",
        description: "Six-character tile label for the module upgrade. 'Mod' abbreviates the game's module system.",
    },
    "score.improvements.operator.tag.mod3.desc": {
        text: "Module L3",
        description: "Name of the module upgrade. 'Module' is the game's equipment system and 'L3' its third level.",
    },
    "score.improvements.operator.tag.mod3.detail": {
        text: "raising one advanced module to Level 3",
        description: "Mid-sentence continuation of the module tooltip. An 'advanced' module is one beyond the basic X module. Lowercase on purpose.",
    },
    "score.improvements.operator.tag.pot6.label": {
        text: "Pot",
        description: "Six-character tile label for the potential upgrade. 'Pot' abbreviates the game's potential system.",
    },
    "score.improvements.operator.tag.pot6.desc": {
        text: "Potential 6",
        description: "Name of the potential upgrade. 'Potential' is the game's duplicate-operator progression system.",
    },
    "score.improvements.operator.tag.pot6.detail": {
        text: "reaching Potential 6",
        description: "Mid-sentence continuation of the potential tooltip. Lowercase on purpose.",
    },
    "score.improvements.operator.tag.trust.label": {
        text: "Trust",
        description: "Six-character tile label for the trust upgrade. 'Trust' is the game's own affinity stat.",
    },
    "score.improvements.operator.tag.trust.desc": {
        text: "Max trust",
        description: "Name of the trust upgrade.",
    },
    "score.improvements.operator.tag.trust.detail": {
        text: "reaching the trust target (200% for published support units, 100% otherwise)",
        description: "Mid-sentence continuation of the trust tooltip. A 'published support unit' is one lent out through the game's support system. Lowercase on purpose.",
    },
    "score.improvements.operator.tag.tooltip": {
        text: "{desc} - {detail}: {value}",
        description: "Native tooltip on an upgrade tag: the upgrade's name, what it involves, then either the gain or the negligible note.",
    },
    "score.improvements.operator.tag.tooltip.gain": {
        text: "+{section}% to this section (+{overall} to overall grade)",
        description: "The value half of an upgrade-tag tooltip when the gain is measurable.",
    },
    "score.improvements.operator.tag.tooltip.negligible": {
        text: "less than 0.01% - negligible",
        description: "The value half of an upgrade-tag tooltip when the gain rounds away.",
    },
    "score.improvements.operator.dim.title": {
        text: "Where your score comes from",
        description: "Heading over the rows that decompose the Operator subscore into investment axes.",
    },
    "score.improvements.operator.dim.earned": {
        text: "{pct}% earned",
        description: "Count badge beside that heading: how much of the section the account has earned so far.",
    },
    "score.improvements.operator.dim.rowTitle": {
        text: "{label}: worth {share}% of this section. You've completed {completion}% of that, earning {earned}%.",
        description: "Native tooltip on one investment-axis row, for an axis with no explanatory line.",
    },
    "score.improvements.operator.dim.rowTitleDetail": {
        text: "{label} - {detail}: worth {share}% of this section. You've completed {completion}% of that, earning {earned}%.",
        description: "Native tooltip on one investment-axis row. {detail} is the axis's lowercase explanation.",
    },
    "score.improvements.operator.dim.footnote.unit": {
        text: "earned / available",
        description: "The highlighted unit in that footnote: each row shows what is earned against what is available.",
    },
    "score.improvements.operator.dim.footnote": {
        text: "Each row is one investment axis: {unit}, both in % of this section. What an axis is worth depends on your roster (rarer operators and their masteries/modules count for more); the earned figures add up to the section score at the top of the card.",
        description: "Footnote under the investment axes. {unit} is the highlighted 'earned / available' (score.improvements.operator.dim.footnote.unit) and may move wherever the sentence needs it.",
    },
    "score.improvements.operator.dim.elite.label": {
        text: "Elites",
        description: "Investment axis: Elite promotions. 'Elite' is the game's promotion tier.",
    },
    "score.improvements.operator.dim.elite.tooltip": {
        text: "promotion progress toward each operator's max elite",
        description: "Explanation of the Elites axis. Appears mid-sentence after the axis name, hence lowercase.",
    },
    "score.improvements.operator.dim.level.label": {
        text: "Levels",
        description: "Investment axis: operator levels.",
    },
    "score.improvements.operator.dim.level.tooltip": {
        text: "level progress across all elite phases",
        description: "Explanation of the Levels axis. Appears mid-sentence after the axis name, hence lowercase.",
    },
    "score.improvements.operator.dim.mastery.label": {
        text: "Masteries",
        description: "Investment axis: skill masteries. 'Mastery' is the game's skill-rank system.",
    },
    "score.improvements.operator.dim.mastery.tooltip": {
        text: "skill masteries, anchored on M3 milestones",
        description: "Explanation of the Masteries axis. 'M3' is the game's abbreviation for Mastery 3. Appears mid-sentence, hence lowercase.",
    },
    "score.improvements.operator.dim.skillLevel.label": {
        text: "Skill levels",
        description: "Investment axis: shared skill levels.",
    },
    "score.improvements.operator.dim.skillLevel.tooltip": {
        text: "skill levels toward SL7 on operators without masteries",
        description: "Explanation of the Skill levels axis. 'SL7' is the game's abbreviation for skill level 7. Appears mid-sentence, hence lowercase.",
    },
    "score.improvements.operator.dim.module.label": {
        text: "Modules",
        description: "Investment axis: operator modules. 'Module' is the game's equipment system.",
    },
    "score.improvements.operator.dim.module.tooltip": {
        text: "advanced module levels, anchored on Mod3 milestones",
        description: "Explanation of the Modules axis. 'Mod3' is the game's abbreviation for module level 3. Appears mid-sentence, hence lowercase.",
    },
    "score.improvements.operator.dim.potential.label": {
        text: "Potential",
        description: "Investment axis: operator potential. 'Potential' is the game's duplicate-operator system.",
    },
    "score.improvements.operator.dim.potential.tooltip": {
        text: "potential on operators where dupes are scarce",
        description: "Explanation of the Potential axis. 'Dupes' are duplicate copies of an operator. Appears mid-sentence, hence lowercase.",
    },
    "score.improvements.operator.dim.trust.label": {
        text: "Trust",
        description: "Investment axis: operator trust. 'Trust' is the game's own affinity stat.",
    },
    "score.improvements.operator.dim.trust.tooltip": {
        text: "trust toward 100% (200% for published support units)",
        description: "Explanation of the Trust axis. A 'published support unit' is one lent out through the game's support system. Appears mid-sentence, hence lowercase.",
    },
    "score.improvements.operator.bucket.toUpgrade": {
        text: "to upgrade",
        description: "Follows the count on a rarity bucket header: '12 to upgrade'.",
    },
    "score.improvements.operator.bucket.gainTitle": {
        text: "Maxing every {rarity}★ below milestone adds ~{gain}% to this section's score.",
        description: "Native tooltip on a rarity bucket's gain badge. {rarity} is the star rating.",
    },
    "score.improvements.operator.row.gainTitle": {
        text: "Completing every upgrade on {name} would add ~{section}% to this section (~{overall} to your overall grade). ELITE/MAX_LEVEL overlap is deduped.",
        description: "Native tooltip on one operator's gain badge. {name} is the operator's name from the game data; ELITE and MAX_LEVEL are raw API tag codes and stay as-is.",
    },
} satisfies MessageMap;

// `dynamic`: the tag and axis keys are stored in the lookup tables above and
// resolved as `t(TAG_LABEL[tag])`, so the extractor has no literal call site to
// match them against.
export const { keys } = defineMessages({ namespace, messages, dynamic: true });
