import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tools";

export const messages = {
    "randomizer.hero.title": {
        text: "Randomizer",
        description: "Page heading of the squad randomizer.",
    },
    "randomizer.hero.intro": {
        text: "Roll a random stage, squad, and challenge modifier. Logged-in users can restrict the draw to operators they own and stages they've completed.",
        description: "Blurb under the page heading. 'Roll' and 'draw' are dice metaphors for picking at random.",
    },
    "randomizer.hero.roster": {
        text: "Roster ",
        description: "Label before the number of operators in the chosen roster. Keep the trailing space; the number follows in bold.",
    },
    "randomizer.hero.drawableOps": {
        text: "Drawable ops ",
        description: "Label before the number of operators the randomizer can currently pick. 'Ops' is short for operators; the space is narrow. Keep the trailing space.",
    },
    "randomizer.hero.drawableStages": {
        text: "Drawable stages ",
        description: "Label before the number of stages the randomizer can currently pick. Keep the trailing space; the number follows in bold.",
    },
    "randomizer.hero.roll": {
        text: "Roll squad",
        description: "Main button: pick a stage, a squad and a challenge at random.",
    },
    "randomizer.hero.reset": {
        text: "Reset",
        description: "Button that clears the current roll.",
    },
    "randomizer.hero.settings": {
        text: "Settings",
        description: "Button that opens the panel constraining what can be drawn.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
