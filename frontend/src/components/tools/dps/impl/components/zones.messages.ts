import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

/**
 * Zone names, stacking rules, supporter scopes and the "which multiplier is this?" table.
 * `zones.ts` and the helper table are plain data that carry these KEYS; the component
 * that renders a row resolves it with `t()`.
 */
export const namespace = "tools";

export const messages = {
    "dps.zone.atkPercent": {
        text: "ATK +%",
        description: "Buff input: percentage attack bonus that adds up with other percentage bonuses. 'ATK' is the game's abbreviation.",
    },
    "dps.zone.inspire": {
        text: "Inspiration",
        description: "Buff input: flat attack granted after percentages, usually a share of another operator's ATK. 'Inspiration' is the game's own name for this buff type.",
    },
    "dps.zone.aspd": {
        text: "ASPD +",
        description: "Buff input: attack speed bonus in points. 'ASPD' is the game's abbreviation.",
    },
    "dps.zone.spRecovery": {
        text: "SP per second +",
        description: "Buff input: extra skill points recovered per second, adding up with other such bonuses. 'SP' is the game's abbreviation.",
    },
    "dps.zone.fragile": {
        text: "Damage taken +%",
        description: "Debuff input: the enemy takes this much more damage of every type. The community calls it 'Fragile'; only the strongest one counts.",
    },
    "dps.zone.defPercent": {
        text: "DEF down %",
        description: "Debuff input: enemy defense lowered by a percentage. 'DEF' is the game's abbreviation.",
    },
    "dps.zone.defFlat": {
        text: "DEF down",
        description: "Debuff input: enemy defense lowered by a fixed number.",
    },
    "dps.zone.resPercent": {
        text: "RES down %",
        description: "Debuff input: enemy arts resistance lowered by a percentage. 'RES' is the game's abbreviation.",
    },
    "dps.zone.resFlat": {
        text: "RES down",
        description: "Debuff input: enemy arts resistance lowered by a fixed number of points.",
    },
    "dps.zone.atkFlat": {
        text: "Base ATK +",
        description: "Buff input: a fixed attack bonus added BEFORE percentage bonuses, so those percentages also scale it.",
    },
    "dps.zone.atkMultiplier": {
        text: "Final ATK ×",
        description: "Buff input: a multiplier applied to attack after every other bonus. The × sign stays.",
    },
    "dps.zone.spAura": {
        text: "SP aura per second",
        description: "Buff input: an SP-per-second aura from another operator; only the strongest aura counts.",
    },
    "dps.zone.physicalFragile": {
        text: "Physical damage taken +%",
        description: "Debuff input: the enemy takes this much more physical damage; only the strongest one counts.",
    },
    "dps.zone.artsFragile": {
        text: "Arts damage taken +%",
        description: "Debuff input: the enemy takes this much more arts damage; only the strongest one counts. 'Arts' is the game's magic damage type.",
    },
    "dps.zone.damageReduction": {
        text: "Enemy damage reduction %",
        description: "Input: the enemy's own reduction of all damage it takes (common on bosses).",
    },
    "dps.zone.defIgnoreFlat": {
        text: "Ignore DEF",
        description: "Input: the attacker ignores this many points of enemy defense.",
    },
    "dps.zone.defIgnorePercent": {
        text: "Ignore DEF %",
        description: "Input: the attacker ignores this percentage of enemy defense.",
    },
    "dps.zone.defSteal": {
        text: "DEF steal",
        description: "Debuff input: defense taken from the enemy by an effect that does not stack with itself; only the strongest counts.",
    },
    "dps.zone.resIgnoreFlat": {
        text: "Ignore RES",
        description: "Input: the attacker ignores this many points of enemy arts resistance.",
    },
    "dps.zone.resIgnorePercent": {
        text: "Ignore RES %",
        description: "Input: the attacker ignores this percentage of enemy arts resistance.",
    },
    "dps.zone.resSteal": {
        text: "RES steal",
        description: "Debuff input: arts resistance taken from the enemy by an effect that does not stack with itself; only the strongest counts.",
    },

    "dps.zone.stacking.additive": {
        text: "adds up",
        description: "How sources of one buff combine: every source is summed.",
    },
    "dps.zone.stacking.max": {
        text: "highest only",
        description: "How sources of one buff combine: only the largest source applies, the rest are ignored.",
    },
    "dps.zone.stacking.multiply": {
        text: "multiplies",
        description: "How sources of one buff combine: each source multiplies the result.",
    },

    "dps.supporter.target.allAllies": {
        text: "All allies",
        description: "Who a supporter's effect reaches: every friendly operator.",
    },
    "dps.supporter.target.alliesInRange": {
        text: "Allies in range",
        description: "Who a supporter's effect reaches: friendly operators inside the supporter's attack range.",
    },
    "dps.supporter.target.oneAlly": {
        text: "One ally",
        description: "Who a supporter's effect reaches: a single friendly operator.",
    },
    "dps.supporter.target.enemies": {
        text: "Enemies",
        description: "Who a supporter's effect reaches: enemies (a debuff).",
    },

    "dps.helper.atkPercent.wording": {
        text: "“ATK +40%”",
        description: "Example of game text the helper explains. Keep the number; use the game's own wording for an attack percentage bonus in your language.",
    },
    "dps.helper.atkPercent.note": {
        text: "Most skill and talent ATK buffs. Several of them add together.",
        description: "Helper explanation for percentage attack buffs.",
    },
    "dps.helper.inspire.wording": {
        text: "“gains ATK equal to 30% of this unit's ATK”",
        description: "Example of game text for an Inspiration buff. Use the game's own wording in your language.",
    },
    "dps.helper.inspire.note": {
        text: "Inspiration. Added after percentages, and only the strongest one counts.",
        description: "Helper explanation for Inspiration buffs.",
    },
    "dps.helper.atkFlat.wording": {
        text: "“ATK +60” (a fixed number)",
        description: "Example of game text for a fixed attack bonus that percentages then scale.",
    },
    "dps.helper.atkFlat.note": {
        text: "A flat bonus that percentage buffs then scale, like a module's base stat.",
        description: "Helper explanation for flat attack added before percentages.",
    },
    "dps.helper.atkMultiplier.wording": {
        text: "“ATK multiplied by 1.2” after everything else",
        description: "Example of game text for a final attack multiplier.",
    },
    "dps.helper.atkMultiplier.note": {
        text: "Rare. Scales the finished ATK, Inspiration included.",
        description: "Helper explanation for the final attack multiplier.",
    },
    "dps.helper.aspd.wording": {
        text: "“ASPD +30”",
        description: "Example of game text for an attack speed bonus.",
    },
    "dps.helper.aspd.note": {
        text: "Attack speed points. They add together.",
        description: "Helper explanation for attack speed buffs.",
    },
    "dps.helper.sp.wording": {
        text: "“recovers 0.3 SP per second”",
        description: "Example of game text for an SP regeneration effect.",
    },
    "dps.helper.sp.note": {
        text: "SP per second. An aura from another operator that does not stack goes in SP aura.",
        description: "Helper explanation for SP recovery versus SP aura.",
    },
    "dps.helper.fragile.wording": {
        text: "“takes 30% more damage”",
        description: "Example of game text for a damage-taken increase on an enemy.",
    },
    "dps.helper.fragile.note": {
        text: "Fragile. Only the strongest counts; physical-only and arts-only versions are separate and multiply with it.",
        description: "Helper explanation for Fragile and typed fragility.",
    },
    "dps.helper.def.wording": {
        text: "“DEF -25%” or “DEF -150”",
        description: "Example of game text for an enemy defense reduction, as a percentage or a number.",
    },
    "dps.helper.def.note": {
        text: "DEF down, percent or flat. The same goes for RES.",
        description: "Helper explanation for defense and resistance reduction.",
    },
    "dps.helper.ignore.wording": {
        text: "“attacks ignore 40% of the target's DEF”",
        description: "Example of game text for ignoring part of a target's defense.",
    },
    "dps.helper.ignore.note": {
        text: "Ignore DEF. Applied after every DEF down.",
        description: "Helper explanation for defense ignore.",
    },
    "dps.helper.steal.wording": {
        text: "“steals 70 DEF”",
        description: "Example of game text for an effect that takes defense from an enemy.",
    },
    "dps.helper.steal.note": {
        text: "DEF steal. Like DEF down, but only the strongest steal counts.",
        description: "Helper explanation for defense steal.",
    },
    "dps.helper.reduction.wording": {
        text: "“the boss takes 50% less damage”",
        description: "Example of game text for an enemy's own damage reduction.",
    },
    "dps.helper.reduction.note": {
        text: "The enemy's own resistance to all damage. Each source multiplies.",
        description: "Helper explanation for enemy damage reduction.",
    },
} satisfies MessageMap;

// `dynamic`: zone labels, stacking words, scopes and helper rows are stored on data
// entries and resolved as `t(entry.key)`, so the extractor has no literal call site.
export const { keys } = defineMessages({ namespace, messages, dynamic: true });
