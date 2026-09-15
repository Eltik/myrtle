import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "user";

export const messages = {
    "profile.base.headline.efficiency": {
        text: "Efficiency",
        description: "Headline figure: the combined production efficiency of the base. Rendered uppercase by CSS.",
    },
    "profile.base.headline.efficiency.hint": {
        text: "Production rooms",
        description: "Sub-line under the efficiency figure, saying which rooms it covers.",
    },
    "profile.base.headline.lmd": {
        text: "LMD / day",
        description: "Headline figure: currency produced per day. 'LMD' is the in-game currency and stays as-is.",
    },
    "profile.base.headline.exp": {
        text: "EXP / day",
        description: "Headline figure: experience produced per day. 'EXP' is the game's own abbreviation.",
    },
    "profile.base.headline.power": {
        text: "Power",
        description: "Headline figure: the base's net electricity. Rendered uppercase by CSS.",
    },
    "profile.base.headline.power.hint": {
        text: "{generated} generated · {consumed} drawn",
        description: "Sub-line under the power figure: what the plants make and what the rooms draw. Keep the middle dot.",
    },
    "profile.base.reset": {
        text: "Reset to my base",
        description: "Button that throws away the planned layout and goes back to the account's real staffing.",
    },
    "profile.base.optimizing": {
        text: "Optimizing…",
        description: "Button label while the optimizer is running. Ends with an ellipsis character.",
    },
    "profile.base.reoptimize": {
        text: "Re-optimize",
        description: "Button label that runs the optimizer again over an already-planned board.",
    },
    "profile.base.optimize": {
        text: "Optimize",
        description: "Button label that runs the optimizer for the first time.",
    },
    "profile.base.evaluationError": {
        text: "This plan could not be scored: {error}",
        description: "Error banner over the board. {error} is the message the server returned.",
    },
    "profile.base.optimizeError": {
        text: "The optimizer failed: {error}",
        description: "Error banner over the board when the optimize request itself failed. {error} is the server's message.",
    },
    "profile.base.fullscreen": {
        text: "Open board in full screen",
        description: "Accessible name of the button that blows the base board up to fill the window.",
    },
    "profile.base.boardTitle": {
        text: "RIIC base board",
        description: "Screen-reader title of the full-screen board. 'RIIC' is the game's own name for the base system.",
    },
    "profile.base.zoomOut": {
        text: "Zoom out",
        description: "Accessible name of the full-screen board's zoom-out button.",
    },
    "profile.base.zoomIn": {
        text: "Zoom in",
        description: "Accessible name of the full-screen board's zoom-in button.",
    },
    "profile.base.resetView": {
        text: "Reset board view",
        description: "Accessible name of the button that restores the full-screen board's zoom and position.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
