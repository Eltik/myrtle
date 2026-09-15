import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tools";

export const messages = {
    "randomizer.stage.kicker": {
        text: "STAGE",
        description: "Vertical label down the side of the stage panel. Set in capitals with wide letter spacing, so keep it very short.",
    },
    "randomizer.stage.target": {
        text: "Target",
        description: "Small heading over the drawn stage, calling it the mission target.",
    },
    "randomizer.stage.rerollAria": {
        text: "Reroll stage",
        description: "Accessible name of the button that draws a new stage.",
    },
    "randomizer.stage.reroll": {
        text: "Reroll",
        description: "Visible label of the button that draws a new stage.",
    },
    "randomizer.stage.family": {
        text: " · {family}",
        description: "The broader event or zone a stage belongs to, shown faintly after its own zone name. Keep the leading space and middle dot.",
    },
    "randomizer.stage.ap": {
        text: "AP {cost}",
        description: "Badge showing the stage's energy cost. 'AP' is the game's own abbreviation for the sanity/stamina it spends.",
    },
    "randomizer.stage.dangerLevel": {
        text: "LV {level}",
        description: "Badge showing the stage's recommended level. 'LV' is the game's own abbreviation.",
    },
    "randomizer.stage.boss": {
        text: "Boss",
        description: "Badge marking a stage that has a boss enemy.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
