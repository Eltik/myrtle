import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "stages";

export const messages = {
    "props.title": {
        text: "Properties",
        description: "Kicker over the list of yes/no stage flags.",
    },
    "props.canPractice": {
        text: "Can Practice",
        description: "Stage flag: the operation can be run in practice mode, which costs no Sanity.",
    },
    "props.battleReplay": {
        text: "Battle Replay",
        description: "Stage flag: a clear of this operation can be replayed.",
    },
    "props.autoDeploy": {
        text: "Auto-Deploy (Multi)",
        description: "Stage flag: the operation supports the game's multi-run auto-deploy. Keep the parenthetical short.",
    },
    "props.storyOnly": {
        text: "Story Only",
        description: "Stage flag: the operation is a story scene with no battle.",
    },
    "props.predefinedSquad": {
        text: "Predefined Squad",
        description: "Stage flag: the operation hands you a fixed roster.",
    },
    "props.predefinedSquad.hint": {
        text: "The stage hands you a fixed roster of operators to clear it with, instead of letting you bring your own.",
        description: "Tooltip on the Predefined Squad flag.",
    },
    "props.trainingLevel": {
        text: "Training Level",
        description: "Stage flag: the operation is a tutorial / practice level.",
    },
    "props.trainingLevel.hint": {
        text: "A tutorial / practice stage that teaches a mechanic and does not count toward normal progression.",
        description: "Tooltip on the Training Level flag.",
    },
    "props.steeringEnabled": {
        text: "Steering Enabled",
        description: "Stage flag: the level file's enemy-pathfinding steering option.",
    },
    "props.steeringEnabled.hint": {
        text: "A pathfinding flag: when on, enemies steer smoothly around each other and corners rather than snapping tile-to-tile. It rarely affects strategy.",
        description: "Tooltip on the Steering Enabled flag.",
    },
    "props.cardsSelectable": {
        text: "Cards Selectable",
        description: "Stage flag: in a predefined-squad operation, whether the player picks which provided operators to deploy.",
    },
    "props.cardsSelectable.hint": {
        text: "In a predefined-squad stage, whether you may pick which of the provided operators to deploy. When off, the loadout is locked.",
        description: "Tooltip on the Cards Selectable flag.",
    },
    "props.ids.title": {
        text: "Identifiers",
        description: "Kicker over the grid of raw backend ids for this stage.",
    },
    "props.ids.stage": {
        text: "Stage ID",
        description: "Label over the stage's raw id. The value is a game-data identifier and is never translated.",
    },
    "props.ids.level": {
        text: "Level ID",
        description: "Label over the stage's raw level-file id.",
    },
    "props.ids.zone": {
        text: "Zone ID",
        description: "Label over the raw id of the zone the stage belongs to.",
    },
    "props.ids.challenge": {
        text: "Challenge ID",
        description: "Label over the raw id of this stage's Challenge Mode counterpart.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
