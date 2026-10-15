import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "user";

export const messages = {
    "profile.showcase.tile.operatorAria": {
        text: "{name}: show {player}'s build",
        description: "Accessible name of an operator tile in a showcase favourites block; activating it opens a dialog with that player's build of the operator. {name} is the operator, {player} the profile's player name.",
    },
    "profile.showcase.tile.factionAria": {
        text: "{name}: show its operators",
        description: "Accessible name of a faction tile in a showcase favourites block; activating it opens a dialog listing every operator of the faction. {name} is the faction.",
    },
    "profile.showcase.operator.private": {
        text: "{player}'s roster is private, so their build of this operator isn't shown.",
        description: "Dialog text when a visitor opens an operator favourite but the player hid their Roster tab. {player} is the player's name.",
    },
    "profile.showcase.operator.notOwned": {
        text: "Not in {player}'s roster.",
        description: "Dialog text when a player's operator favourite is one they do not own. {player} is the player's name.",
    },
    "profile.showcase.operator.openPage": {
        text: "Open operator page",
        description: "Link in a showcase dialog to the operator's own page on the site.",
    },
    "profile.showcase.operator.loading": {
        text: "Loading {name}'s build",
        description: "Accessible name of the placeholder shown while a player's operator build loads. {name} is the operator.",
    },
    "profile.showcase.faction.count": {
        text: "{count, plural, one {# operator} other {# operators}}",
        description: "How many operators a faction holds, in the faction dialog of a showcase favourite.",
    },
    "profile.showcase.faction.ownedCount": {
        text: "{owned} of {count} owned",
        description: "How many of a faction's operators the player owns, in the faction dialog. {owned} and {count} are numbers.",
    },
    "profile.showcase.faction.ownedOnly": {
        text: "Owned only",
        description: "Switch in the faction dialog that narrows the list to the operators the player owns.",
    },
    "profile.showcase.faction.owned": {
        text: "Owned",
        description: "Badge on an operator card in the faction dialog: the player owns this operator.",
    },
    "profile.showcase.faction.private": {
        text: "{player}'s roster is private, so ownership isn't shown.",
        description: "Note in the faction dialog when the player's Roster tab is hidden from the viewer. {player} is the player's name.",
    },
    "profile.showcase.faction.empty": {
        text: "No operator in this server's data belongs to this faction.",
        description: "Shown in the faction dialog when the game data the reader uses lists no operator for the faction (a faction whose operators are not released on this server).",
    },
    "profile.showcase.faction.emptyOwned": {
        text: "{player} owns none of these operators.",
        description: "Shown in the faction dialog when 'Owned only' is on and the player owns none of the faction's operators.",
    },
    "profile.showcase.faction.gridLabel": {
        text: "Operators of {name}",
        description: "Accessible name of the grid of operator cards in the faction dialog. {name} is the faction.",
    },
    "profile.showcase.faction.cardBuild": {
        text: "{name}: show {player}'s build",
        description: "Accessible name of an owned operator card in the faction dialog; it opens the player's build. {name} is the operator.",
    },
    "profile.showcase.faction.cardPage": {
        text: "{name}: open operator page",
        description: "Accessible name of an operator card in the faction dialog that links to the operator's page.",
    },
    "profile.showcase.faction.unobtainable": {
        text: "Not obtainable",
        description: "Badge on an operator card in the faction dialog for a form the game never hands out on its own (an alternate form, a reserve operator).",
    },
    "profile.showcase.faction.upcoming": {
        text: "Upcoming: not yet on this server ({count})",
        description: "Heading over the operators of a faction that are out on CN but not yet on the reader's server, the ones /operators lists under Upcoming. {count} is how many.",
    },
    "profile.showcase.faction.upcomingAria": {
        text: "{name}: upcoming, open operator page",
        description: "Accessible name of an upcoming (not yet released on this server) operator card in the faction dialog. {name} is the operator.",
    },
    "profile.showcase.player": {
        text: "Player {uid}",
        description: "Stand-in for a player's name in showcase dialogs when the profile has no nickname. {uid} is the game uid.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
