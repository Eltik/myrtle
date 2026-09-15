import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tools";

export const messages = {
    "calc.chart.allHidden.title": {
        text: "All curves hidden",
        description: "Empty-state heading in place of the chart when every plotted operator has been toggled off.",
    },
    "calc.chart.allHidden.desc": {
        text: "Toggle visibility on a card (or use the chart legend) to plot it again.",
        description: "Empty-state body telling the reader how to bring a hidden curve back.",
    },
    "calc.chart.ariaLabel": {
        text: "{yLabel} by {xLabel} for {count, plural, one {# operator} other {# operators}}",
        description: "Screen-reader description of the chart. {yLabel} and {xLabel} are the axis names, e.g. 'Skill DPS by Enemy DEF for 2 operators'.",
    },
    "calc.chart.calculating": {
        text: "Calculating",
        description: "Live status shown over the chart while results are still being fetched.",
    },
    "calc.chart.snapshot": {
        text: "snapshot",
        description: "Tiny label on the dashed vertical line marking the X value the result panel reports. Lowercase on purpose.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
