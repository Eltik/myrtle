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
    "planner.plansLoadFailed": {
        text: "Couldn't load your plans.",
        description: "Shown in place of the plan list when the request for the user's plans failed, beside a retry button.",
    },
    "planner.requirementsLoadFailed": {
        text: "Couldn't load the materials these plans need.",
        description: "Shown in place of the requirements panel when the request for the selected plans' combined materials failed, beside a retry button.",
    },
    "planner.rosterLoadFailed": {
        text: "Couldn't load your roster, so progress toward each plan isn't shown.",
        description: "Shown above the plan list when the user's synced roster failed to load, beside a retry button. Plan progress is measured against the roster.",
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
    "planner.filter.aria": {
        text: "Filter plans by group",
        description: "Accessible name of the row of group chips above the plan list.",
    },
    "planner.filter.all": {
        text: "All",
        description: "Chip that clears the group filter so every plan counts toward the requirements.",
    },
    "planner.filter.ungrouped": {
        text: "Ungrouped",
        description: "Chip that selects the plans not sorted into any group.",
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
        text: "Couldn't delete. Try again.",
        description: "Error shown in the delete dialog when the request failed.",
    },
    "planner.unknownOperator": {
        text: "Unknown operator",
        description: "Stands in for a plan's operator name when the operator could not be resolved.",
    },
    "planner.group.none": {
        text: "No groups created yet.",
        description: "Shown in place of the group list when the player has made none.",
    },
    "planner.group.empty": {
        text: "No plans in this group.",
        description: "Shown inside an expanded group that has no plans in it.",
    },
    "planner.group.pin": {
        text: "Pin {name}",
        description: "Accessible name and tooltip of the button that pins a group to the top of the list; {name} is the player's own group name.",
    },
    "planner.group.unpin": {
        text: "Unpin {name}",
        description: "Accessible name and tooltip of the button that unpins a pinned group; {name} is the player's own group name.",
    },
    "planner.group.expand": {
        text: "Show plans in {name}",
        description: "Accessible name of the chevron button that expands a collapsed group to list its plans; {name} is the player's own group name.",
    },
    "planner.group.collapse": {
        text: "Hide plans in {name}",
        description: "Accessible name of the chevron button that collapses an expanded group; {name} is the player's own group name.",
    },
    "planner.group.rename": {
        text: "Rename {name}",
        description: "Accessible name of the pencil button that renames a group; {name} is the player's own group name.",
    },
    "planner.group.delete": {
        text: "Delete {name}",
        description: "Accessible name of the bin button that deletes a group; {name} is the player's own group name.",
    },
    "planner.card.expand": {
        text: "Show plan details",
        description: "Accessible name of the chevron button that expands a plan card to show its level, skill and module targets.",
    },
    "planner.card.collapse": {
        text: "Hide plan details",
        description: "Accessible name of the chevron button that collapses an expanded plan card.",
    },
    "planner.card.moduleNone": {
        text: "None",
        description: "Screen-reader text for a module stage of 0 in a plan card's progress rows (shown visually as a hyphen): the module is not unlocked now, or is left out of the target.",
    },
    "planner.completed.check": {
        text: "Completed plans",
        description: "Header button that lists the plans whose targets the last roster sync shows as reached, whether or not the notice is showing.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
