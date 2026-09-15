import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tools";

export const messages = {
    "planner.breadcrumb.tools": {
        text: "Tools",
        description: "First crumb of the breadcrumb trail, naming the section this tool lives in.",
    },
    "planner.title": {
        text: "Operator Planner",
        description: "Page heading and last breadcrumb.",
    },
    "planner.intro": {
        text: "Plan your operator promotion, level, skill, and module goals.",
        description: "Blurb under the page heading. 'Promotion' and 'module' are the game's own progression systems.",
    },
    "planner.signIn.title": {
        text: "Sign in to use the planner",
        description: "Heading of the card shown to a visitor who is not signed in.",
    },
    "planner.signIn.desc": {
        text: "Your operator promotion, level, skill, and module goals are saved to your account so you can access them anywhere.",
        description: "Body of the card shown to a visitor who is not signed in.",
    },
    "planner.signIn.action": {
        text: "Sign in",
        description: "Button that opens the sign-in dialog.",
    },
    "planner.createPlan": {
        text: "Create plan",
        description: "Button that opens the dialog for adding a new operator plan.",
    },
    "planner.createFirstPlan": {
        text: "Create new plan",
        description: "Large button shown in place of the empty plan list.",
    },
    "planner.tab.plans": {
        text: "Plans",
        description: "Tab listing every saved operator plan.",
    },
    "planner.tab.groups": {
        text: "Groups",
        description: "Tab listing the named groups plans can be sorted into.",
    },
    "planner.selectAll": {
        text: "Select all",
        description: "Checkbox label that includes every plan in the requirement totals.",
    },
    "planner.unselectAll": {
        text: "Unselect all",
        description: "Checkbox label that drops every plan from the requirement totals.",
    },
    "planner.deleteSelected": {
        text: "Delete selected ({count})",
        description: "Button that deletes every selected plan; the count is how many that is.",
    },
    "planner.card.level": {
        text: "Level",
        description: "Row label in an expanded plan card, above the current and target promotion and level.",
    },
    "planner.card.rarityClass": {
        text: "{rarity}★ {archetype}",
        description: "Second line of a plan card: star rating then archetype, e.g. '6★ Centurion'. {archetype} is game vocabulary and comes from the game data.",
    },
    "planner.card.eliteAlt": {
        text: "Elite {elite}",
        description: "Alt text of a promotion icon. 'Elite' is the game's promotion tier and {elite} is 0, 1 or 2.",
    },
    "planner.card.levelValue": {
        text: "Lv.{level}",
        description: "An operator level, e.g. 'Lv.90'. 'Lv.' is the game's own abbreviation.",
    },
    "planner.card.skills": {
        text: "Skills",
        description: "Section label in an expanded plan card, above the per-skill current and target ranks.",
    },
    "planner.card.skillFallback": {
        text: "Skill {index}",
        description: "Stands in for a skill's name when the game data has none; {index} is the skill's 1-based slot.",
    },
    "planner.card.modules": {
        text: "Modules",
        description: "Section label in an expanded plan card, above the per-module current and target stages. 'Module' is the game's own equipment system.",
    },
    "planner.card.mastery": {
        text: "M{mastery}",
        description: "A mastery rank, e.g. 'M3'. The game's own notation, which normally stays as-is.",
    },
    "planner.card.edit": {
        text: "Edit",
        description: "Button on a plan card that opens it for editing.",
    },
    "planner.card.delete": {
        text: "Delete",
        description: "Button on a plan card that deletes that plan.",
    },
    "planner.deleteFailed": {
        text: "Failed to delete. Please try again.",
        description: "Error shown in the delete dialog when the request failed.",
    },
    "planner.unknownOperator": {
        text: "Unknown operator",
        description: "Stands in for a plan's operator name when the operator could not be resolved.",
    },
    "planner.group.renamePrompt": {
        text: "Enter new group name:",
        description: "Prompt in the browser dialog that renames a group. Keep the colon.",
    },
    "planner.group.deleteConfirm": {
        text: 'Are you sure you want to delete group "{name}"?',
        description: "Question in the browser dialog confirming a group deletion; {name} is the group's own name, in quotes.",
    },
    "planner.group.none": {
        text: "No groups created yet.",
        description: "Shown in place of the group list when the player has made none.",
    },
    "planner.group.empty": {
        text: "No plans in this group.",
        description: "Shown inside an expanded group that has no plans in it.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
