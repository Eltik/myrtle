import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tools";

export const messages = {
    "squad.stage.empty": {
        text: "No stage picked. Pick one to use its deploy limit and starting DP, and to see its enemies.",
        description: "Stage card in the Squad Builder before a stage is chosen. DP is deployment points, the in-game cost currency.",
    },
    "squad.stage.pick": {
        text: "Pick a stage",
        description: "Button that opens the stage picker.",
    },
    "squad.stage.change": {
        text: "Change",
        description: "Button that opens the stage picker to swap the picked stage.",
    },
    "squad.stage.open": {
        text: "Open stage",
        description: "Link to the stage's own page.",
    },
    "squad.stage.remove": {
        text: "Remove",
        description: "Button that unpicks the stage.",
    },
    "squad.stage.missing": {
        text: "Couldn't load this stage.",
        description: "Shown in the stage card when the stage could not be loaded.",
    },
    "squad.stage.normal": {
        text: "Normal",
        description: "Toggle option for the normal version of a stage.",
    },
    "squad.stage.challenge": {
        text: "Challenge",
        description: "Toggle option for the challenge (hard) version of a stage.",
    },
    "squad.stage.mode": {
        text: "Stage version",
        description: "Accessible name of the Normal / Challenge toggle.",
    },
    "squad.stage.sanity": {
        text: "Sanity",
        description: "Fact label: the stage's sanity cost (the in-game stamina currency).",
    },
    "squad.stage.deployLimit": {
        text: "Deploy limit",
        description: "Fact label: how many operators the stage lets you deploy.",
    },
    "squad.stage.startDp": {
        text: "Starting DP",
        description: "Fact label: deployment points at the start of the stage.",
    },
    "squad.stage.dpRate": {
        text: "DP per second",
        description: "Fact label: how fast deployment points come in.",
    },
    "squad.stage.lifePoints": {
        text: "Life points",
        description: "Fact label: how many enemies may get through before the stage is lost.",
    },
    "squad.stage.enemies": {
        text: "Enemies",
        description: "Fact label: the number of enemies the stage spawns.",
    },
    "squad.stage.none": {
        text: "None",
        description: "Shown for a stage fact the stage does not have (no limit, no number).",
    },
    "squad.stage.threats": {
        text: "What its enemies ask for",
        description: "Heading over the threat chips: kinds of enemy in this stage that need a specific answer.",
    },
    "squad.stage.noThreats": {
        text: "Nothing that needs a special answer.",
        description: "Shown under the threat heading when none of the stage's enemies stand out.",
    },
    "squad.stage.threat.high_res": {
        text: "High RES",
        description: "Threat chip: enemies with high arts resistance. RES is the in-game abbreviation.",
    },
    "squad.stage.threat.high_def": {
        text: "High DEF",
        description: "Threat chip: enemies with high defense. DEF is the in-game abbreviation.",
    },
    "squad.stage.threat.aerial": {
        text: "Aerial",
        description: "Threat chip: flying enemies.",
    },
    "squad.stage.threat.invisible": {
        text: "Invisible",
        description: "Threat chip: enemies that can't be targeted until revealed.",
    },
    "squad.stage.threat.multi_life": {
        text: "Costs extra life points",
        description: "Threat chip: enemies that take more than one life point when they get through.",
    },
    "squad.stage.threat.fast": {
        text: "Fast",
        description: "Threat chip: enemies that move quickly.",
    },
    "squad.stage.threatCount": {
        text: "{label} x{count}",
        description: "A threat chip with how many enemies carry it, e.g. 'Aerial x4'.",
    },
    "squad.stage.showEnemies": {
        text: "Show enemies",
        description: "Button that reveals the stage's enemy list.",
    },
    "squad.stage.hideEnemies": {
        text: "Hide enemies",
        description: "Button that hides the stage's enemy list.",
    },
    "squad.stage.enemyCount": {
        text: "x{count}",
        description: "How many of an enemy the stage spawns, e.g. 'x3'.",
    },
    "squad.stage.dpValue": {
        text: "{value}/s",
        description: "DP per second value, e.g. '1/s'.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
