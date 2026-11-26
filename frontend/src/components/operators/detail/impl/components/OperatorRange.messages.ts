import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "operators";

export const messages = {
    "range.aria": {
        text: "Attack range: {tiles, plural, one {# tile} other {# tiles}}, reaching {ahead, plural, one {# tile} other {# tiles}} ahead, {behind, plural, one {# behind} other {# behind}} and {side, plural, one {# to each side} other {# to each side}}",
        description: "Screen-reader description of the attack-range grid. The operator faces right; 'ahead' counts tiles in the facing direction, 'behind' the opposite, 'to each side' the widest reach up or down. {tiles} is how many tiles the operator can hit. Every number may be 0.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
