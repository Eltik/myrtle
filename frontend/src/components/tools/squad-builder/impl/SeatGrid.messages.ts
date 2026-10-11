import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tools";

export const messages = {
    "squad.grid.aria": {
        text: "Squad seats",
        description: "Accessible name of the grid of twelve squad seats in the Squad Builder.",
    },
    "squad.grid.leader": {
        text: "Leader",
        description: "Tag on the first seat of a squad. The first operator in an Arknights squad is its leader.",
    },
    "squad.grid.seatTag": {
        text: "Seat {seat}",
        description: "Tag at the top of a seat card. {seat} is the seat number, 1 to 12.",
    },
    "squad.grid.add": {
        text: "Add",
        description: "Label inside an empty squad seat; clicking it opens the operator picker.",
    },
    "squad.grid.addAria": {
        text: "Add an operator to seat {seat}",
        description: "Accessible name of an empty seat. {seat} is the seat number, 1 to 12.",
    },
    "squad.grid.seatAria": {
        text: "Seat {seat}: {name}, edit",
        description: "Accessible name of a filled seat card; activating it opens the seat editor. {name} is the operator's name (game text).",
    },
    "squad.grid.level": {
        text: "Lv {level}",
        description: "Operator level on a seat card, e.g. 'Lv 90'. Keep it short.",
    },
    "squad.grid.rarityAria": {
        text: "{rarity} star",
        description: "Accessible name of the rarity stars on a seat card. {rarity} is 1 to 6.",
    },
    "squad.grid.eliteAria": {
        text: "Elite {phase}",
        description: "Accessible name of the promotion icon on a seat card. {phase} is 0, 1 or 2.",
    },
    "squad.grid.potentialAria": {
        text: "Potential {rank}",
        description: "Accessible name of the potential icon on a seat card. {rank} is 1 to 6.",
    },
    "squad.grid.skillAria": {
        text: "Skill {skill}",
        description: "Accessible name of the small skill icon on a seat card. {skill} is 1, 2 or 3.",
    },
    "squad.grid.skillShort": {
        text: "S{skill}",
        description: "Short label of the chosen skill on a seat card, e.g. 'S2'. {skill} is 1, 2 or 3. Keep it to two or three characters.",
    },
    "squad.grid.skillLevelAria": {
        text: "Skill level {level}",
        description: "Accessible name of the small numeral on the skill tile of a seat card when the skill has no mastery yet. {level} is 1 to 7.",
    },
    "squad.grid.masteryAria": {
        text: "Mastery {mastery}",
        description: "Accessible name of the mastery glyph over the skill icon on a seat card. {mastery} is 1 to 3.",
    },
    "squad.grid.moduleRank": {
        text: "Rank {rank}",
        description: "Module rank on the module tile of a seat card, 1 to 3. Shown as a tooltip; the tile itself shows the bare number.",
    },
    "squad.grid.noSkill": {
        text: "No skill",
        description: "On a seat card when the operator has no skill at all (1 and 2 star operators).",
    },
    "squad.grid.noModule": {
        text: "No module",
        description: "On a seat card when the operator has no module equipped.",
    },
    "squad.grid.menu": {
        text: "Seat options",
        description: "Accessible name of the '...' button on a seat card that opens its menu.",
    },
    "squad.grid.makeLeader": {
        text: "Make leader",
        description: "Seat menu item: swap this operator into the first (leader) seat.",
    },
    "squad.grid.move": {
        text: "Move to seat...",
        description: "Seat menu item: pick another seat number to swap with. The keyboard alternative to dragging.",
    },
    "squad.grid.remove": {
        text: "Remove",
        description: "Seat menu item: take the operator out of the squad.",
    },
    "squad.grid.openDps": {
        text: "Open in DPS calculator",
        description: "Seat menu item: open the DPS calculator with this operator.",
    },
    "squad.grid.moveTitle": {
        text: "Move {name} to which seat?",
        description: "Title of the dialog that moves a seat by number. {name} is the operator's name (game text).",
    },
    "squad.grid.moveDesc": {
        text: "Whoever sits there swaps places with them.",
        description: "Description in the move-to-seat dialog.",
    },
    "squad.grid.moveSeat": {
        text: "Seat {seat}",
        description: "A seat number button in the move-to-seat dialog.",
    },
    "squad.grid.support": {
        text: "Support",
        description: "Tag at the top of the support seat card: an operator borrowed from a friend for one run.",
    },
    "squad.grid.supportCaption": {
        text: "Borrowed from a friend at their full build",
        description: "Caption under the support seat card.",
    },
    "squad.grid.supportAdd": {
        text: "Pick a support",
        description: "Label inside the empty support seat card.",
    },
    "squad.grid.supportAria": {
        text: "Support: {name}, edit",
        description: "Accessible name of the filled support card. {name} is the operator's name (game text).",
    },
    "squad.grid.dragHint": {
        text: "Drag a card onto another to swap them.",
        description: "Small hint under the seat grid.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
