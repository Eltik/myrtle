import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

/**
 * The bulk-add dialog: one target applied to many operator plans, plus saved
 * presets of that target.
 *
 * The Elite (`E2`), mastery (`M3`) and skill/module designators are the game's
 * own shorthand. Labels it shares with the single plan dialog (promotion,
 * Elite N, stage, cancel, the operator search) come from that dialog's file.
 */
export const namespace = "tools";

export const messages = {
    "planner.bulk.open": {
        text: "Bulk add",
        description: "Button beside the add-plan button that opens the dialog for planning many operators at once.",
    },
    "planner.bulk.title": {
        text: "Bulk add plans",
        description: "Title of the dialog that plans many operators with one target.",
    },
    "planner.bulk.desc": {
        text: "Give several operators the same target. Each one stops at what its rarity allows.",
        description: "Subtitle of the bulk dialog. 'Rarity' is the operator's star rating.",
    },
    "planner.bulk.operators": {
        text: "Operators",
        description: "Label over the multi-select operator picker.",
    },
    "planner.bulk.selectedCount": {
        text: "{count, plural, one {# selected} other {# selected}}",
        description: "Beside the operator picker's label: how many operators are picked.",
    },
    "planner.bulk.addAllResults": {
        text: "{count, plural, one {Add the # result} other {Add all # results}}",
        description: "Row at the top of the operator picker that adds every operator matching the search.",
    },
    "planner.bulk.clear": {
        text: "Clear",
        description: "Button that removes every picked operator.",
    },
    "planner.bulk.remove": {
        text: "Remove {operator}",
        description: "Accessible name of the x on one picked operator. {operator} is an operator name from the game data.",
    },
    "planner.bulk.target": {
        text: "Target",
        description: "Heading over the target fields (promotion, level, skills, modules) applied to every picked operator.",
    },
    "planner.bulk.level": {
        text: "Level",
        description: "Label over the target level field.",
    },
    "planner.bulk.levelMax": {
        text: "Max",
        description: "Switch beside the level field: when on, each operator targets the highest level at its promotion.",
    },
    "planner.bulk.levelCapNote": {
        text: "Up to {max} here. Lower rarities stop at their own cap.",
        description: "Hint under the level field. {max} is the highest level the chosen promotion allows for any operator.",
    },
    "planner.bulk.skillLevel": {
        text: "Skill level",
        description: "Label over the shared skill level buttons (1 to 7).",
    },
    "planner.bulk.masteries": {
        text: "Masteries",
        description: "Label over the per-skill mastery rows. 'Mastery' is the game's skill upgrade past level 7.",
    },
    "planner.bulk.skillSlot": {
        text: "S{index}",
        description: "Short name of a skill slot, e.g. 'S1' for the first skill. Game shorthand.",
    },
    "planner.bulk.moduleStage": {
        text: "Module stage",
        description: "Label over the module stage buttons. 'Module' is the game's operator equipment.",
    },
    "planner.bulk.targetNote": {
        text: "Missing skills and modules are skipped. Anything already past the target stays where it is.",
        description: "Hint under the target fields: an operator with fewer skills or no module ignores those rows, and a plan never goes below the operator's current progress.",
    },
    "planner.bulk.preset": {
        text: "Preset",
        description: "Label over the saved-preset picker.",
    },
    "planner.bulk.presetPlaceholder": {
        text: "Load a preset",
        description: "Placeholder of the preset picker before one is chosen.",
    },
    "planner.bulk.presetNone": {
        text: "No saved presets yet.",
        description: "Shown in the preset picker when the player has saved none.",
    },
    "planner.bulk.presetName": {
        text: "Preset name",
        description: "Placeholder of the field naming a new preset.",
    },
    "planner.bulk.presetSave": {
        text: "Save preset",
        description: "Button that saves the current target as a preset under the typed name.",
    },
    "planner.bulk.presetReplace": {
        text: "Replace preset",
        description: "The save-preset button when the typed name matches a saved preset, which it overwrites.",
    },
    "planner.bulk.presetDelete": {
        text: "Delete preset",
        description: "Accessible name of the button that deletes the chosen preset.",
    },
    "planner.bulk.overwrite": {
        text: "Replace existing plans",
        description: "Switch label: when on, operators that already have a plan get the new target.",
    },
    "planner.bulk.overwrite.desc": {
        text: "When off, operators that already have a plan are skipped.",
        description: "Hint under the replace-existing-plans switch.",
    },
    "planner.bulk.preview": {
        text: "Preview",
        description: "Heading over the list of picked operators and the target each will get.",
    },
    "planner.bulk.previewEmpty": {
        text: "Pick operators to see the target each one gets.",
        description: "Shown in the preview while no operator is picked.",
    },
    "planner.bulk.status.loading": {
        text: "Loading...",
        description: "Preview row status while that operator's data loads. Three full stops, not an ellipsis character.",
    },
    "planner.bulk.status.planned": {
        text: "Skipped: already planned",
        description: "Preview row status: the operator has a plan and replacing is off.",
    },
    "planner.bulk.status.reached": {
        text: "Skipped: already there",
        description: "Preview row status: the operator's current progress meets or passes the whole target.",
    },
    "planner.bulk.status.unplannable": {
        text: "Skipped: can't be planned",
        description: "Preview row status for an operator whose upgrades the planner does not support.",
    },
    "planner.bulk.status.replaces": {
        text: "Replaces plan",
        description: "Preview row tag: this operator's existing plan will be overwritten.",
    },
    "planner.bulk.summary.promotion": {
        text: "E{elite} Lv{level}",
        description: "Compact promotion and level in a preview row, e.g. 'E2 Lv90'. Game shorthand.",
    },
    "planner.bulk.summary.skill": {
        text: "SL{level}",
        description: "Compact skill level in a preview row, e.g. 'SL7'. Game shorthand.",
    },
    "planner.bulk.summary.masteries": {
        text: "M{masteries}",
        description: "Compact masteries per skill in a preview row, e.g. 'M3/3/0'. {masteries} is the slash-separated list.",
    },
    "planner.bulk.summary.module": {
        text: "Mod {stage}",
        description: "Compact module stage in a preview row, e.g. 'Mod 3'.",
    },
    "planner.bulk.save": {
        text: "{count, plural, =0 {Add plans} one {Add # plan} other {Add # plans}}",
        description: "Button that saves a plan for every operator not skipped.",
    },
    "planner.bulk.failed": {
        text: "{count, plural, one {# plan was not saved.} other {# plans were not saved.}} The reason is on each row.",
        description: "Error banner after saving when some operators failed. The operators that saved are already removed from the list.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
