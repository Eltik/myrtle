import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

/**
 * Labels for the derived values `helpers.ts` produces. That module has no
 * React, so its label functions take a `t` and fall back to the source English
 * for the one caller that is not inside a provider.
 *
 * Skill and talent TEXT is absent on purpose: it comes from the game data,
 * which ships its own translation per region.
 */
export const namespace = "operators";

export const messages = {
    "attr.atk": {
        text: "ATK",
        description: "Module stat row: attack power. The game's own abbreviation, which normally stays as-is.",
    },
    "attr.maxHp": {
        text: "HP",
        description: "Module stat row: max health. The game's own abbreviation, which normally stays as-is.",
    },
    "attr.def": {
        text: "DEF",
        description: "Module stat row: defense. The game's own abbreviation, which normally stays as-is.",
    },
    "attr.attackSpeed": {
        text: "ASPD",
        description: "Module stat row: attack speed. The game's own abbreviation, which normally stays as-is.",
    },
    "attr.magicResistance": {
        text: "RES",
        description: "Module stat row: arts resistance. The game's own abbreviation, which normally stays as-is.",
    },
    "attr.cost": {
        text: "DP Cost",
        description: "Module stat row: deployment cost. 'DP' is the in-battle currency and stays as-is.",
    },
    "attr.respawnTime": {
        text: "Redeploy",
        description: "Module stat row: the cooldown before the operator can be placed again.",
    },
    "attr.blockCnt": {
        text: "Block",
        description: "Module stat row: how many enemies the operator holds.",
    },
    "attr.hpRecovery": {
        text: "HP Regen",
        description: "Module stat row: health regenerated per second. 'HP' stays as-is; 'Regen' is short for regeneration.",
    },
    "attr.spRecovery": {
        text: "SP Regen",
        description: "Module stat row: skill points regenerated per second. 'SP' is the game's skill-point abbreviation and stays as-is.",
    },
    "attr.baseAttackTime": {
        text: "Attack Interval",
        description: "Module stat row: seconds between attacks.",
    },
    "skill.level.lv": {
        text: "Lv.{level}",
        description: "Skill level 1 to 7, e.g. 'Lv.4'. Very tight space - abbreviate the word for level the way the game does.",
    },
    "skill.level.mastery": {
        text: "M{level}",
        description: "Skill mastery rank 1 to 3, e.g. 'M3'. The game's own shorthand, which normally stays as-is.",
    },
    "skill.spType.autoRecovery": {
        text: "Auto Recovery",
        description: "How a skill charges: skill points build up over time.",
    },
    "skill.spType.offensiveRecovery": {
        text: "Offensive Recovery",
        description: "How a skill charges: skill points build up when the operator attacks.",
    },
    "skill.spType.defensiveRecovery": {
        text: "Defensive Recovery",
        description: "How a skill charges: skill points build up when the operator is hit.",
    },
    "skill.spType.onDeployment": {
        text: "On Deployment",
        description: "How a skill charges: it is ready the moment the operator is placed.",
    },
    "skill.type.passive": {
        text: "Passive",
        description: "How a skill fires: always on, never activated.",
    },
    "skill.type.manual": {
        text: "Manual Trigger",
        description: "How a skill fires: the player taps it once charged.",
    },
    "skill.type.auto": {
        text: "Auto Trigger",
        description: "How a skill fires: it goes off by itself once charged.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
