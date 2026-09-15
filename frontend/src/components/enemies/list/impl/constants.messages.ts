import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

/**
 * The option lists and enum label tables in `constants.ts` (and the damage
 * palette in `tokens.ts`) are plain data in modules with no React, so they
 * carry message KEYS and whichever component renders a row resolves it with
 * `t()`.
 *
 * Enemy names, callsigns, races and handbook text are game vocabulary and come
 * from the game-data layer, so they are deliberately absent.
 */
export const namespace = "enemies";

export const messages = {
    "sort.index": {
        text: "Index",
        description: "Sort option: by the enemy's in-game handbook index (its callsign, e.g. 'B1'). Short label in a compact select.",
    },
    "sort.name": {
        text: "Name",
        description: "Sort option: alphabetically by enemy name. Short label in a compact select.",
    },
    "sort.level": {
        text: "Threat",
        description: "Sort option: by threat tier (Normal, then Elite, then Boss). Short label in a compact select.",
    },
    "sort.hp": {
        text: "HP",
        description: "Sort option: by hit points. Abbreviated to fit a compact select.",
    },
    "sort.atk": {
        text: "ATK",
        description: "Sort option: by attack. Abbreviated to fit a compact select.",
    },
    "sort.def": {
        text: "DEF",
        description: "Sort option: by defense. Abbreviated to fit a compact select.",
    },
    "sort.res": {
        text: "RES",
        description: "Sort option: by magic resistance. Abbreviated to fit a compact select.",
    },
    "sort.weight": {
        text: "Weight",
        description: "Sort option: by mass level, which decides how far the enemy can be shoved. Short label in a compact select.",
    },
    "level.NORMAL": {
        text: "Normal",
        description: "Threat tier: an ordinary enemy. Also the filter chip for that tier.",
    },
    "level.ELITE": {
        text: "Elite",
        description: "Threat tier: a tougher, named variant. Shown as a short uppercase badge, so keep it brief.",
    },
    "level.BOSS": {
        text: "Boss",
        description: "Threat tier: a boss enemy. Shown as a short uppercase badge, so keep it brief.",
    },
    "damageType.PHYSIC": {
        text: "Physical",
        description: "Damage type an enemy deals: ordinary physical damage, mitigated by DEF.",
    },
    "damageType.MAGIC": {
        text: "Arts",
        description: "Damage type an enemy deals: magic damage, mitigated by RES. 'Arts' is the game's own name for magic; keep the game's term.",
    },
    "damageType.HEAL": {
        text: "Heal",
        description: "Damage type an enemy deals: it heals rather than harms.",
    },
    "damageType.NO_DAMAGE": {
        text: "No Damage",
        description: "Damage type an enemy deals: none - it does not attack.",
    },
    "applyWay.MELEE": {
        text: "Melee",
        description: "Attack range of an enemy: it strikes adjacent tiles only.",
    },
    "applyWay.RANGED": {
        text: "Ranged",
        description: "Attack range of an enemy: it strikes at a distance.",
    },
    "applyWay.NONE": {
        text: "None",
        description: "Attack range of an enemy: it has no attack at all.",
    },
} satisfies MessageMap;

// `dynamic`: these keys are stored on constants / token entries and resolved by
// the consuming component as `t(item.labelKey)`, so the extractor has no
// literal call site to match them against.
export const { keys } = defineMessages({ namespace, messages, dynamic: true });
