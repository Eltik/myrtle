import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tools";

export const messages = {
    "squad.picker.title": {
        text: "Add to seat {seat}",
        description: "Heading of the operator picker that opens under the grid when an empty seat is clicked. {seat} is 1 to 12.",
    },
    "squad.picker.supportTitle": {
        text: "Pick a support",
        description: "Heading of the operator picker for the support seat (an operator borrowed from a friend).",
    },
    "squad.picker.close": {
        text: "Close",
        description: "Button that closes the operator picker.",
    },
    "squad.picker.search": {
        text: "Search operators",
        description: "Placeholder and accessible name of the search box in the operator picker.",
    },
    "squad.picker.owned": {
        text: "Owned only",
        description: "Switch in the operator picker that hides operators the signed-in player does not have.",
    },
    "squad.picker.class": {
        text: "Class",
        description: "Accessible name of the class filter in the operator picker.",
    },
    "squad.picker.rarity": {
        text: "Rarity",
        description: "Accessible name of the rarity filter in the operator picker.",
    },
    "squad.picker.all": {
        text: "All",
        description: "Filter option that shows every class or every rarity.",
    },
    "squad.picker.stars": {
        text: "{rarity}★",
        description: "A rarity filter option, e.g. '6★'.",
    },
    "squad.picker.branch": {
        text: "Branch",
        description: "Accessible name of the branch (sub-class, e.g. 'Centurion') filter, shown once a class is chosen in the operator picker.",
    },
    "squad.picker.allBranches": {
        text: "All branches",
        description: "Branch filter option that shows every branch of the chosen class.",
    },
    "squad.picker.sort": {
        text: "Sort",
        description: "Accessible name of the sort menu in the operator picker.",
    },
    "squad.picker.sort.rarity": {
        text: "Rarity",
        description: "Sort option: highest rarity first, then by name.",
    },
    "squad.picker.sort.name": {
        text: "Name",
        description: "Sort option: alphabetical by operator name.",
    },
    "squad.picker.sort.cost": {
        text: "DP cost",
        description: "Sort option: cheapest deployment cost (DP) first.",
    },
    "squad.picker.sort.level": {
        text: "Level",
        description: "Sort option: highest promotion and level in the player's roster first.",
    },
    "squad.picker.seated": {
        text: "Seat {seat}",
        description: "Badge on an operator tile that is already in the squad. {seat} is the seat number, 1 to 12.",
    },
    "squad.picker.seatedSupport": {
        text: "Support",
        description: "Badge on an operator tile that is already the support.",
    },
    "squad.picker.level": {
        text: "Lv {level}",
        description: "Roster level on an owned operator's tile, e.g. 'Lv 90'.",
    },
    "squad.picker.elite": {
        text: "Elite {phase}",
        description: "Accessible name of the promotion icon on an owned operator's tile. {phase} is 0 to 2.",
    },
    "squad.picker.potential": {
        text: "Potential {rank}",
        description: "Accessible name of the potential icon on an owned operator's tile. {rank} is 1 to 6.",
    },
    "squad.picker.skill": {
        text: "S{skill}",
        description: "Short label of the skill the player has selected for an owned operator, e.g. 'S3' for skill 3.",
    },
    "squad.picker.skillTitle": {
        text: "Skill {skill}, rank {level}",
        description: "Tooltip of the skill label on an owned operator's tile. {skill} is 1 to 3, {level} is the skill rank 1 to 7.",
    },
    "squad.picker.mastery": {
        text: "Mastery {mastery}",
        description: "Accessible name of the mastery icon next to the skill label on an owned operator's tile. {mastery} is 1 to 3.",
    },
    "squad.picker.module": {
        text: "{type} {rank}",
        description: "The module an owned operator has equipped, as its type tag and rank, e.g. 'MAR-X 3'. {type} is game text.",
    },
    "squad.picker.moduleTitle": {
        text: "Module {type}, rank {rank}",
        description: "Tooltip of the module label on an owned operator's tile.",
    },
    "squad.picker.cost": {
        text: "{cost} DP",
        description: "Deployment cost on an operator tile, e.g. '20 DP'. DP is the in-battle deployment points currency.",
    },
    "squad.picker.costTitle": {
        text: "Deployment cost at the highest promotion, before potential",
        description: "Tooltip of the DP cost on an operator tile, saying which cost the number is.",
    },
    "squad.picker.currentForm": {
        text: "In use",
        description: "Badge on the form of an operator with several forms (e.g. Amiya as a Caster, Guard or Medic) that the player has set in game.",
    },
    "squad.picker.notOwned": {
        text: "Not owned",
        description: "On an operator tile when the signed-in player does not have that operator.",
    },
    "squad.picker.pick": {
        text: "Add {name}, {branch}",
        description: "Accessible name of an operator tile in the picker. {name} is the operator's name and {branch} the branch, e.g. 'Add Amiya, Incantation Medic' (both game text).",
    },
    "squad.picker.count": {
        text: "{count, plural, one {# operator} other {# operators}}",
        description: "Result count above the operator picker grid.",
    },
    "squad.picker.hint.move": {
        text: "move",
        description: "Keyboard hint after the arrow-key glyphs: the arrow keys move between operator tiles.",
    },
    "squad.picker.hint.pick": {
        text: "add",
        description: "Keyboard hint after the 'Enter' key: Enter adds the focused operator.",
    },
    "squad.picker.hint.close": {
        text: "close",
        description: "Keyboard hint after the 'Esc' key: Escape closes the picker.",
    },
    "squad.picker.empty": {
        text: "No operators match.",
        description: "Shown in the picker when the search and filters leave nothing.",
    },
    "squad.picker.clearFilters": {
        text: "Clear filters",
        description: "Button under 'No operators match.' that resets the search and every filter in the picker.",
    },
    "squad.picker.loading": {
        text: "Loading operators...",
        description: "Shown in the picker while the operator list loads.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
