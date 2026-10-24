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
    "calc.chart.marker.break": {
        text: "floor from {x}",
        description: "Curve marker detail: the axis value from which the operator's damage stops falling because every hit has reached the 5% minimum. {x} is a formatted axis value such as 'DEF 1,240', or the 'not in range' text. Lowercase on purpose; it follows the operator name.",
    },
    "calc.chart.marker.half": {
        text: "half at {x}",
        description: "Curve marker detail: the axis value where the curve falls to half of its value at the start of the sweep. {x} is a formatted axis value such as 'DEF 860', or the 'not in range' text. Lowercase on purpose.",
    },
    "calc.chart.marker.notInRange": {
        text: "not in range",
        description: "Stands in for the axis value in a curve-marker detail when the curve never reaches that point inside the swept range.",
    },
    "calc.chart.marker.breakKey": {
        text: "Floor reached: damage stops falling",
        description: "Legend entry under the chart explaining the filled diamond marker: from here on every hit deals the 5% minimum, so more DEF or RES no longer lowers the value.",
    },
    "calc.chart.marker.halfKey": {
        text: "Half of the value at the sweep start",
        description: "Legend entry under the chart explaining the hollow ring marker: where the curve falls to half of its value at the left edge of the chart.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
