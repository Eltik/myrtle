import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

/**
 * The axis, metric and enemy-preset tables in `constants.ts` are plain data in
 * a module with no React, so they carry message KEYS and the component that
 * renders a row resolves it with `t()`.
 */
export const namespace = "tools";

export const messages = {
    "dps.metric.skillDps": {
        text: "Skill DPS",
        description: "Y-axis metric: damage per second while the skill is running. 'DPS' is the usual abbreviation and stays as-is.",
    },
    "dps.metric.averageDps": {
        text: "Average DPS",
        description: "Y-axis metric: damage per second averaged over skill uptime and recharge.",
    },
    "dps.metric.totalDamage": {
        text: "Total Damage",
        description: "Y-axis metric: all damage dealt over one skill activation.",
    },
    "dps.metric.skillDps.short": {
        text: "Skill DPS",
        description: "Column heading for skill DPS in the result panel. Very narrow column.",
    },
    "dps.metric.averageDps.short": {
        text: "Avg DPS",
        description: "Column heading for average DPS in the result panel. Abbreviated because the column is very narrow.",
    },
    "dps.metric.totalDamage.short": {
        text: "Total",
        description: "Column heading for total damage in the result panel. Very narrow column, so a single word.",
    },
    "dps.axis.defense": {
        text: "Enemy DEF",
        description: "X-axis name: the target's defense. 'DEF' is the game's own abbreviation and stays as-is.",
    },
    "dps.axis.res": {
        text: "Enemy RES (%)",
        description: "X-axis name: the target's arts resistance as a percentage. 'RES' is the game's own abbreviation.",
    },
    "dps.axis.defense.short": {
        text: "DEF",
        description: "Compact label for the defense axis tab on narrow screens. The game's own abbreviation.",
    },
    "dps.axis.res.short": {
        text: "RES %",
        description: "Compact label for the resistance axis tab on narrow screens. The game's own abbreviation.",
    },
    "dps.preset.trash": {
        text: "Trash",
        description: "Enemy preset: an ordinary weak enemy. Community jargon for the cheap enemies that arrive in waves.",
    },
    "dps.preset.trash.summary": {
        text: "DEF 0 · RES 0",
        description: "Tooltip listing the stats the 'Trash' preset applies. 'DEF' and 'RES' are the game's abbreviations.",
    },
    "dps.preset.elite": {
        text: "Elite",
        description: "Enemy preset: a tougher mid-tier enemy. 'Elite' is the game's own enemy tier.",
    },
    "dps.preset.elite.summary": {
        text: "DEF 1000 · RES 0",
        description: "Tooltip listing the stats the 'Elite' preset applies.",
    },
    "dps.preset.armored": {
        text: "Armored",
        description: "Enemy preset: a high-defense enemy.",
    },
    "dps.preset.armored.summary": {
        text: "DEF 1800 · RES 20",
        description: "Tooltip listing the stats the 'Armored' preset applies.",
    },
    "dps.preset.boss": {
        text: "Boss",
        description: "Enemy preset: a stage boss. 'Boss' is the game's own enemy tier.",
    },
    "dps.preset.boss.summary": {
        text: "DEF 2500 · RES 30",
        description: "Tooltip listing the stats the 'Boss' preset applies.",
    },
    "dps.preset.casterTrash": {
        text: "Caster trash",
        description: "Enemy preset: weak enemies with high arts resistance instead of armor. 'Caster' is the game's arts-using class.",
    },
    "dps.preset.casterTrash.summary": {
        text: "DEF 0 · RES 60",
        description: "Tooltip listing the stats the 'Caster trash' preset applies.",
    },
    "dps.preset.droneWave": {
        text: "Drone wave",
        description: "Enemy preset: several flying drones at once. 'Drone' is the game's own enemy type.",
    },
    "dps.preset.droneWave.summary": {
        text: "DEF 200 · 3 targets",
        description: "Tooltip listing the stats the 'Drone wave' preset applies, including how many enemies are hit at once.",
    },
} satisfies MessageMap;

// `dynamic`: these keys are stored on a constants entry and resolved by the
// consuming component as `t(item.labelKey)`, so the extractor has no literal
// call site to match them against.
export const { keys } = defineMessages({ namespace, messages, dynamic: true });
