import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "user";

export const messages = {
    "score.contribution.title": {
        text: "Grade composition",
        description: "Card title over the bar showing how much each section contributes to the overall grade.",
    },
    "score.contribution.segment": {
        text: "{label}: {earned} of {share} pts",
        description: "Tooltip on one segment of the composition bar. {label} is a section name, {earned} the points earned and {share} the points available; 'pts' is short for points.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
