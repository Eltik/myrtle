import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "operators";

export const messages = {
    "cnUpdates.badge": {
        text: "Updated",
        description: "Short pill on an operator card when the CN server's data for this operator differs from this server's (a new module, changed values). Keep it to one short word.",
    },
    "cnUpdates.badge.title": {
        text: "Updated on CN: {changes}",
        description: "Hover text and accessible name of the Updated pill. {changes} is a comma-separated list of change labels such as 'new module'. 'CN' is the Chinese game server, which is months ahead.",
    },
    "cnUpdates.kind.module_added": {
        text: "{count, plural, one {new module} other {# new modules}}",
        description: "Change label: CN has modules for this operator that this server does not. 'Module' is the game's word for an operator's equipment upgrade.",
    },
    "cnUpdates.kind.module_changed": {
        text: "{count, plural, one {module values} other {# modules' values}}",
        description: "Change label: a module both servers have carries different numbers on CN.",
    },
    "cnUpdates.kind.skill_added": {
        text: "{count, plural, one {new skill} other {# new skills}}",
        description: "Change label: CN lists skills for this operator that this server does not.",
    },
    "cnUpdates.kind.skill_values_changed": {
        text: "{count, plural, one {skill values} other {# skills' values}}",
        description: "Change label: a skill both servers have carries different numbers (SP cost, duration, multipliers) on CN.",
    },
    "cnUpdates.kind.talents_changed": {
        text: "talents",
        description: "Change label: talent numbers or unlock conditions differ on CN. 'Talent' is the game's word for an operator's passive ability.",
    },
    "cnUpdates.kind.trait_changed": {
        text: "trait",
        description: "Change label: the class trait's numbers differ on CN.",
    },
    "cnUpdates.kind.stats_changed": {
        text: "stats",
        description: "Change label: base attributes (HP, ATK, redeployment time and so on) differ on CN.",
    },
    "cnUpdates.kind.range_changed": {
        text: "attack range",
        description: "Change label: the attack range of a promotion differs on CN.",
    },
    "cnUpdates.kind.potentials_changed": {
        text: "potentials",
        description: "Change label: potential bonuses differ on CN. 'Potential' is the game's word for duplicate-copy upgrades.",
    },
    "cnUpdates.kind.trust_bonus_changed": {
        text: "trust bonus",
        description: "Change label: the stat bonus at full trust differs on CN.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
