import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

/**
 * The create/edit dialog for one operator plan.
 *
 * Operator, skill and module NAMES come from the game data, as do the Elite
 * (`E0`), mastery (`M3`) and module-stage designators, so only this dialog's
 * own chrome lives here.
 */
export const namespace = "tools";

export const messages = {
    "planner.dialog.editTitle": {
        text: "Edit plan",
        description: "Dialog title when an existing plan is being changed.",
    },
    "planner.dialog.createTitle": {
        text: "Create new plan",
        description: "Dialog title when a new plan is being added.",
    },
    "planner.dialog.editDesc": {
        text: "Customize and update your target goals for {operator}.",
        description: "Dialog subtitle when editing; {operator} is an operator name from the game data.",
    },
    "planner.dialog.editDesc.fallback": {
        text: "this operator",
        description: "Stands in for the operator name in the editing subtitle while it is still loading.",
    },
    "planner.dialog.createDesc": {
        text: "Add a new operator target plan to your planner list.",
        description: "Dialog subtitle when creating a plan.",
    },
    "planner.dialog.selectOperator": {
        text: "Select Operator",
        description: "Label over the operator picker.",
    },
    "planner.dialog.loadingOperators": {
        text: "Loading operators...",
        description: "Placeholder in the operator picker while the list loads. Three full stops, not an ellipsis character.",
    },
    "planner.dialog.searchOperators": {
        text: "Search operators by name, class, or tag...",
        description: "Placeholder in the operator picker. 'Class' and 'tag' are the game's own operator attributes. Three full stops, not an ellipsis character.",
    },
    "planner.dialog.noOperators": {
        text: "No operators found.",
        description: "Shown inside the operator picker when the query matches nothing.",
    },
    "planner.dialog.rarityClass": {
        text: "{rarity}★ · {class}",
        description: "Second line of an operator row in the picker: star rating then class, e.g. '6★ · Guard'. {class} is game vocabulary and comes from the game data.",
    },
    "planner.dialog.rarityArchetype": {
        text: "{rarity}★ · {archetype}",
        description: "Subtitle of the chosen operator: star rating then archetype, e.g. '6★ · Centurion'. {archetype} is game vocabulary and comes from the game data.",
    },
    "planner.dialog.loadingDetails": {
        text: "Loading operator details...",
        description: "Shown while the chosen operator's data loads. Three full stops, not an ellipsis character.",
    },
    "planner.dialog.radian": {
        text: "Raidian's upgrades are dependent on Integrated Strategies 6 - Sui's Garden of Grotesqueries progression and cannot be planned.",
        description: "Notice for the one operator whose progression this tool cannot model. The operator name and the game mode 'Integrated Strategies 6 - Sui's Garden of Grotesqueries' both come from the game and keep the game's own wording.",
    },
    "planner.dialog.promotion": {
        text: "Promotion",
        description: "Label over the row of Elite promotion buttons. 'Promotion' is the game's own progression step.",
    },
    "planner.dialog.elite": {
        text: "Elite {elite}",
        description: "Native tooltip and alt text of one promotion button. 'Elite' is the game's promotion tier and {elite} is 0, 1 or 2.",
    },
    "planner.dialog.level": {
        text: "Level (Max {max})",
        description: "Label over the level slider, naming the highest level the chosen promotion allows.",
    },
    "planner.dialog.skills": {
        text: "Skills",
        description: "Heading of the collapsible section holding each skill's target rank.",
    },
    "planner.dialog.skillFallback": {
        text: "Skill {index}",
        description: "Stands in for a skill's name when the game data has none; {index} is the skill's 1-based slot.",
    },
    "planner.dialog.sp": {
        text: "SP: {initial}/{total}",
        description: "A skill's starting and required skill points, e.g. 'SP: 10/35'. 'SP' is the game's own abbreviation. Keep the colon.",
    },
    "planner.dialog.masteryLocked": {
        text: "Skill Mastery {mastery} is available after Elite {elite} Level {level}",
        description: "Tooltip on a mastery button the operator has not qualified for. 'Mastery' and 'Elite' are the game's own progression steps.",
    },
    "planner.dialog.skillLevelLocked": {
        text: "Skill Level {skillLevel} is available after Elite {elite} Level {level}",
        description: "Tooltip on a skill-level button the operator has not qualified for. 'Elite' is the game's promotion tier.",
    },
    "planner.dialog.mastery": {
        text: "Mastery {mastery}",
        description: "Native tooltip and alt text of a mastery button, where {mastery} is 1, 2 or 3.",
    },
    "planner.dialog.skillLevel": {
        text: "Level {level}",
        description: "Native tooltip of a pre-mastery skill-level button.",
    },
    "planner.dialog.modules": {
        text: "Modules",
        description: "Heading of the collapsible section holding each module's target stage. 'Module' is the game's own equipment system.",
    },
    "planner.dialog.moduleFallback": {
        text: "Module",
        description: "Stands in for a module's designator when the game data carries none.",
    },
    "planner.dialog.moduleLocked": {
        text: "Module unlocks at Elite {elite} Level {level}",
        description: "Tooltip on a module stage the operator has not qualified for. 'Elite' is the game's promotion tier.",
    },
    "planner.dialog.stage": {
        text: "Stage {stage}",
        description: "Native tooltip of a module-stage button. 'Stage' is the game's own name for a module's upgrade step.",
    },
    "planner.dialog.notPlanned": {
        text: "Not planned",
        description: "Native tooltip of the module-stage button meaning the module is left out of the plan.",
    },
    "planner.dialog.groups": {
        text: "Groups",
        description: "Label over the group picker.",
    },
    "planner.dialog.groups.placeholder": {
        text: "Select plan groups...",
        description: "Placeholder in the group picker. Three full stops, not an ellipsis character.",
    },
    "planner.dialog.groups.newName": {
        text: "New group name...",
        description: "Placeholder in the box for naming a new group. Three full stops, not an ellipsis character.",
    },
    "planner.dialog.groups.create": {
        text: "Create",
        description: "Button that saves the new group being named.",
    },
    "planner.dialog.groups.cancel": {
        text: "Cancel",
        description: "Button that abandons the new group being named.",
    },
    "planner.dialog.groups.createNew": {
        text: "Create new group",
        description: "Link inside the group picker that opens the box for naming a new group.",
    },
    "planner.dialog.groups.none": {
        text: "No groups found.",
        description: "Shown inside the group picker when the query matches nothing.",
    },
    "planner.dialog.groups.renamePrompt": {
        text: "Enter new group name:",
        description: "Prompt in the browser dialog that renames a group. Keep the colon.",
    },
    "planner.dialog.groups.deleteConfirm": {
        text: 'Are you sure you want to delete group "{name}"?',
        description: "Question in the browser dialog confirming a group deletion; {name} is the group's own name, in quotes.",
    },
    "planner.dialog.displayOnProfile": {
        text: "Display on profile",
        description: "Switch label: whether this plan appears on the player's public profile.",
    },
    "planner.dialog.displayOnProfile.desc": {
        text: "Show this target plan on your public user profile.",
        description: "Explanation under the 'Display on profile' switch.",
    },
    "planner.dialog.pickOperator": {
        text: "Please select an operator to customize targets.",
        description: "Shown in place of the plan form before an operator has been chosen. Rendered in italics.",
    },
    "planner.dialog.cancel": {
        text: "Cancel",
        description: "Footer button that closes the dialog without saving.",
    },
    "planner.dialog.saving": {
        text: "Saving...",
        description: "Footer button label while the plan is being saved. Three full stops, not an ellipsis character.",
    },
    "planner.dialog.save": {
        text: "Save changes",
        description: "Footer button that saves an edited plan.",
    },
    "planner.dialog.create": {
        text: "Create plan",
        description: "Footer button that saves a new plan.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
