import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tools";

export const messages = {
    "calc.kpi.title": {
        text: "Snapshot",
        description: "Heading of the panel listing each operator's result at the single enemy setup currently configured.",
    },
    "calc.kpi.leaderBy": {
        text: "Leader by ",
        description: "Lead-in beside the Snapshot heading; the metric being ranked on follows in a brighter color. Keep the trailing space.",
    },
    "calc.kpi.buildSummary": {
        text: "{skill} · {rank} · {module}",
        description: "One-line summary of an operator's build: skill name, skill rank (Lv 7 / M3), and module. All three come from elsewhere; only the middle dots are ours.",
    },
    "calc.kpi.failed": {
        text: "Calculation failed",
        description: "Accessible name and fallback tooltip of the warning icon shown when an operator's calculation returned an error.",
    },
    "calc.kpi.leader": {
        text: "Leader",
        description: "Native tooltip on the crown badge marking the highest-scoring operator.",
    },
    "calc.kpi.top": {
        text: "top",
        description: "Text inside the crown badge on the highest-scoring operator. Lowercase, and very short - it sits in a tiny pill.",
    },
    "calc.kpi.moreInfo": {
        text: "{label} - more info",
        description: "Accessible name of a result figure that carries an explanatory tooltip. {label} is the column name, e.g. 'Avg DPS'.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
