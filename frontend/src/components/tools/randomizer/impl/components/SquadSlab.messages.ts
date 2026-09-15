import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tools";

export const messages = {
    "randomizer.squad.kicker": {
        text: "SQUAD",
        description: "Vertical label down the side of the squad panel. Set in capitals with wide letter spacing, so keep it very short.",
    },
    "randomizer.squad.deployment": {
        text: "Deployment ·",
        description: "Small heading over the drawn squad; the filled-of-total count follows after a space. 'Deployment' is the game's word for the squad you take into a stage.",
    },
    "randomizer.squad.count": {
        text: "{filled}/{size}",
        description: "How many squad slots the draw filled, out of the requested size, e.g. '9/12'.",
    },
    "randomizer.squad.rerollAria": {
        text: "Reroll squad",
        description: "Accessible name of the button that draws a new squad.",
    },
    "randomizer.squad.reroll": {
        text: "Reroll",
        description: "Visible label of the button that draws a new squad.",
    },
    "randomizer.squad.empty": {
        text: "No matching operators - relax your filters.",
        description: "Shown in place of the squad when the filters leave nothing to draw. The dash is a plain hyphen.",
    },
    "randomizer.squad.viewOperator": {
        text: "View {name}",
        description: "Accessible name of an operator tile, which links to that operator's page. {name} comes from the game data.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
