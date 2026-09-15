import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "stages";

export const messages = {
    "overview.title": {
        text: "Overview",
        description: "Kicker over the row of stage stat cards (starting DP, DP rate, unit limit, move speed).",
    },
    "overview.initialCost": {
        text: "Initial DP",
        description: "Stat card label, also the alt text of its icon. 'DP' is the game's Deployment Points; keep the game's own abbreviation.",
    },
    "overview.initialCost.info": {
        text: "The amount of Deployment Points (DP) you start the battle with.",
        description: "Tooltip on the Initial DP stat card. 'Deployment Points' is the in-game term.",
    },
    "overview.dpTick.alt": {
        text: "DP per tick",
        description: "Alt text of the icon on the DP / Tick stat card. Spelled out because a screen reader reads it aloud.",
    },
    "overview.dpTick": {
        text: "DP / Tick",
        description: "Stat card label: how many seconds pass between passive DP gains. Very tight space.",
    },
    "overview.dpTick.info": {
        text: "How often you passively gain 1 DP - a lower number means DP regenerates faster.",
        description: "Tooltip on the DP / Tick stat card.",
    },
    "overview.unitLimit": {
        text: "Unit Limit",
        description: "Stat card label, also the alt text of its icon: how many operators may be deployed at once.",
    },
    "overview.unitLimit.info": {
        text: "The maximum number of operators you can have deployed on the field at once.",
        description: "Tooltip on the Unit Limit stat card.",
    },
    "overview.moveSpeed": {
        text: "Move Speed",
        description: "Stat card label, also the alt text of its icon: the stage's global enemy move-speed multiplier.",
    },
    "overview.moveSpeed.info": {
        text: "A global multiplier applied to every enemy's movement speed on this stage. ×0.5 means enemies move at half their normal speed.",
        description: "Tooltip on the Move Speed stat card. '×0.5' is an example value, not a placeholder.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
