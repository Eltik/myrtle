import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

/**
 * The vocabulary the roster cards share, declared once here next to the
 * helpers they share: the compact card, the detailed card, the unowned card and
 * the operator dialog all render these, and `operatorMissing` in
 * `helpers.card.ts` returns the `gap.*` keys for whoever lists them.
 *
 * Operator, skill and module NAMES come from the game data and are not here.
 * The stat abbreviations, the Elite / Potential / Mastery tiers and 'Trust' are
 * the game's own terms - a translation should follow the game's wording.
 */
export const namespace = "user";

export const messages = {
    "profile.roster.gap.elite": {
        text: "Promote to E{elite}",
        description: "Remaining step on an operator: reach the top Elite promotion. 'E2' is how the game writes its second promotion.",
    },
    "profile.roster.gap.level": {
        text: "Level to {level}",
        description: "Remaining step on an operator: reach the level cap of its current promotion.",
    },
    "profile.roster.gap.mastery": {
        text: "{count, plural, one {# skill} other {# skills}} to M3",
        description: "Remaining step on an operator: how many skills still need Mastery 3. At most three, so the number is inline.",
    },
    "profile.roster.gap.skillLevel": {
        text: "Skill rank to 7",
        description: "Remaining step on an operator with no masteries: raise the shared skill level to its cap of 7.",
    },
    "profile.roster.gap.module": {
        text: "{count, plural, one {# module} other {# modules}} to Lv 3",
        description: "Remaining step on an operator: how many modules still need level 3. A handful at most, so the number is inline.",
    },
    "profile.roster.gap.potential": {
        text: "Max potential (P6)",
        description: "Remaining step on an operator: reach the top potential rank. 'P6' is the game's own shorthand.",
    },
    "profile.roster.gap.trust": {
        text: "Reach 100% trust",
        description: "Remaining step on an operator: raise trust to the 100% milestone.",
    },
    "profile.roster.card.maxed": {
        text: "Maxed",
        description: "Badge on an operator card that has nothing left to invest in. Very little room.",
    },
    "profile.roster.card.level": {
        text: "Level",
        description: "Label over an operator's level.",
    },
    "profile.roster.card.trust": {
        text: "Trust",
        description: "Label over an operator's trust percentage. 'Trust' is the game's own affinity stat.",
    },
    "profile.roster.card.potential": {
        text: "Potential",
        description: "Label over an operator's potential rank. 'Potential' is the game's duplicate-operator system.",
    },
    "profile.roster.card.skills": {
        text: "Skills",
        description: "Heading of the skill list on an operator card.",
    },
    "profile.roster.card.modules": {
        text: "Modules",
        description: "Heading of the module list on an operator card.",
    },
    "profile.roster.card.stat.hp": {
        text: "HP",
        description: "Stat row: max health. The game's own abbreviation, which normally stays as-is.",
    },
    "profile.roster.card.stat.atk": {
        text: "ATK",
        description: "Stat row: attack power. The game's own abbreviation, which normally stays as-is.",
    },
    "profile.roster.card.stat.def": {
        text: "DEF",
        description: "Stat row: defense. The game's own abbreviation, which normally stays as-is.",
    },
    "profile.roster.card.stat.res": {
        text: "RES",
        description: "Stat row: arts resistance. The game's own abbreviation, which normally stays as-is.",
    },
    "profile.roster.card.stat.cost": {
        text: "Cost",
        description: "Stat row: deployment cost.",
    },
    "profile.roster.card.stat.dp": {
        text: "DP",
        description: "Stat row: deployment cost. 'DP' is the in-battle currency and stays as-is.",
    },
    "profile.roster.card.stat.block": {
        text: "Block",
        description: "Stat row: how many enemies the operator holds.",
    },
    "profile.roster.card.noSkills": {
        text: "No skills found.",
        description: "Shown in place of the skill list when the game data has none.",
    },
    "profile.roster.card.noModules": {
        text: "No modules unlocked.",
        description: "Shown in place of the module list when the operator has none unlocked.",
    },
    "profile.roster.card.skillFallback": {
        text: "Skill {n}",
        description: "Stand-in name for a skill the game data does not name; {n} counts from 1.",
    },
    "profile.roster.card.skillLevel": {
        text: "Lv.{level}",
        description: "An operator's skill level on a skill row. 'Lv.' is the game's abbreviation; very little room.",
    },
    "profile.roster.card.moduleLevel": {
        text: "Lv.{level}",
        description: "A module's level on a module row. 'Lv.' is the game's abbreviation; very little room.",
    },
    "profile.roster.card.skillAlt": {
        text: "Skill",
        description: "Alt text of a skill icon, which sits beside the skill's name.",
    },
    "profile.roster.card.moduleAlt": {
        text: "Module",
        description: "Alt text of a module icon, which sits beside the module's name; also the stand-in for an unnamed module type.",
    },
    "profile.roster.card.eliteAlt": {
        text: "Elite {elite}",
        description: "Alt text of the promotion badge. 'Elite' is the game's promotion tier and {elite} is 0, 1 or 2.",
    },
    "profile.roster.card.potentialAlt": {
        text: "Potential {rank}",
        description: "Alt text of the potential badge. {rank} runs from 1 to 6.",
    },
    "profile.roster.card.masteryAlt": {
        text: "M{mastery}",
        description: "Alt text of a mastery badge. 'M3' is the game's own shorthand for Mastery 3 and normally stays as-is.",
    },
    "profile.roster.card.rarityAlt": {
        text: "{star} Star",
        description: "Alt text of the rarity stars. Always written in this one form, whatever the count.",
    },
} satisfies MessageMap;

// `dynamic`: the `gap.*` keys are returned by `operatorMissing` on a plain
// object and resolved by whichever card lists them, so the extractor has no
// literal call site to match them against.
export const { keys } = defineMessages({ namespace, messages, dynamic: true });
