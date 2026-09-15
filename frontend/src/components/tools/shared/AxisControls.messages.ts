import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tools";

export const messages = {
    "calc.axis.xAxis": {
        text: "X axis",
        description: "Label over the tab row that picks which variable the chart sweeps along the horizontal axis.",
    },
    "calc.axis.yMetric": {
        text: "Y metric",
        description: "Label over the select that picks which figure the chart plots on the vertical axis.",
    },
    "calc.axis.metricPlaceholder": {
        text: "Metric",
        description: "Placeholder in the Y-metric select before a metric is chosen.",
    },
    "calc.axis.from": {
        text: "From",
        description: "Label on the number input holding the low end of the swept range.",
    },
    "calc.axis.to": {
        text: "To",
        description: "Label on the number input holding the high end of the swept range.",
    },
    "calc.axis.points": {
        text: "Points",
        description: "Label on the select that sets how many points the chart samples across the range.",
    },
    "calc.axis.pointsPlaceholder": {
        text: "Points",
        description: "Placeholder in the sample-count select before a count is chosen.",
    },
    "calc.axis.pointSummary": {
        text: "{count} points{mode, select, whole { · whole numbers} other { · ~{step}{unit} apart}}",
        description:
            "Caption under the range inputs. {count} is the number of sampled points (any number, including one). The 'whole' branch is used for an axis that only takes integers; otherwise it states the approximate spacing, where {step} is a number and {unit} is an empty string or '%'. The separator is a middle dot.",
    },
    "calc.axis.metricHint": {
        text: "{metric}:",
        description: "Prefix before the one-line explanation of the selected Y metric. {metric} is the metric's own name; keep the colon.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
