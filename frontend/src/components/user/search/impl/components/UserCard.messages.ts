import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "user";

export const messages = {
    "search.card.level": {
        text: "Lv {level}",
        description: "Account level on a search-result card. Abbreviated on purpose; the tile is narrow.",
    },
    "search.card.points": {
        text: "pts",
        description: "Unit after the total score on a search-result card, short for points. The number is rendered just before it.",
    },
    "search.card.operators": {
        text: "ops",
        description: "Unit after the owned-operator count on a search-result card, short for operators. The number is rendered just before it.",
    },
    "search.card.skins": {
        text: "skins",
        description: "Unit after the owned-skin count on a search-result card. The number is rendered just before it.",
    },
    "search.card.metric.operators": {
        text: "{count, plural, one {# operator} other {# operators}}",
        description: "Leading stat on a card while players are ranked by operators owned.",
    },
    "search.card.metric.joined": {
        text: "Joined {date}",
        description: "Leading stat on a card while players are ranked by the date they joined the game. {date} is already formatted.",
    },
    "search.card.metric.enemies": {
        text: "{count, plural, one {# enemy} other {# enemies}}",
        description: "Leading stat on a card while players are ranked by enemies discovered.",
    },
    "search.card.metric.potentials": {
        text: "{count, plural, one {# potential} other {# potentials}}",
        description: "Leading stat on a card while players are ranked by potentials: potential ranks summed over the roster.",
    },
    "search.card.metric.masteries": {
        text: "{count} M3",
        description: "Leading stat on a card while players are ranked by skills at mastery 3. Keep 'M3', the game's shorthand.",
    },
    "search.card.metric.modules": {
        text: "{count, plural, one {# Lv3 module} other {# Lv3 modules}}",
        description: "Leading stat on a card while players are ranked by modules at level 3. Keep 'Lv3'.",
    },
    "search.card.metric.skins": {
        text: "{count, plural, one {# skin} other {# skins}}",
        description: "Leading stat on a card while players are ranked by skins owned.",
    },
    "search.card.metric.scoped": {
        text: "{count, plural, one {# {label}} other {# {label}s}}",
        description: "Leading stat on a card while players are ranked by operators of one class or archetype. {label} is the game's class or archetype name: '23 Guards', '1 Centurion'.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
