import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tools";

export const messages = {
    "squad.slots.title": {
        text: "Your squads",
        description: "Heading over the player's saved squad slots.",
    },
    "squad.slots.signIn": {
        text: "Sign in to save squads. The squad below still works without an account.",
        description: "Shown in place of the saved slots when signed out.",
    },
    "squad.slots.signInButton": {
        text: "Sign in",
        description: "Button that opens the sign-in dialog.",
    },
    "squad.slots.untitled": {
        text: "Squad {slot}",
        description: "Name shown for a saved squad that has no name. {slot} is the slot number, from 1.",
    },
    "squad.slots.count": {
        text: "{count}/{total}",
        description: "How many of the squad's seats a saved squad fills, e.g. '9/12'; {total} is the seats in a squad.",
    },
    "squad.slots.saved": {
        text: "Saved {date}",
        description: "When a squad slot was last saved. {date} is a formatted date.",
    },
    "squad.slots.unsaved": {
        text: "Unsaved",
        description: "Badge on the active slot when the squad has changes that are not saved.",
    },
    "squad.slots.active": {
        text: "Editing",
        description: "Badge on the slot the working squad belongs to.",
    },
    "squad.slots.empty": {
        text: "Empty slot",
        description: "Title of a slot with no squad saved in it.",
    },
    "squad.slots.emptyDesc": {
        text: "Save a squad here",
        description: "Line under 'Empty slot'.",
    },
    "squad.slots.loadAria": {
        text: "Open squad slot {slot}",
        description: "Accessible name of a saved-squad card. {slot} is 1 to 4.",
    },
    "squad.slots.game": {
        text: "In-game squads",
        description: "Heading over the read-only squads brought in from the game by the last account sync.",
    },
    "squad.slots.gameDesc": {
        text: "Read only. Click one to load a copy.",
        description: "Line under the in-game squads heading.",
    },
    "squad.slots.gameEmpty": {
        text: "Sync your account on the Account page to bring in your in-game squads.",
        description: "Shown when there are no in-game squads yet.",
    },
    "squad.slots.gameEmptyLink": {
        text: "Go to Account",
        description: "Link to the account page where the player syncs their game account.",
    },
    "squad.slots.gameSynced": {
        text: "Synced {date}",
        description: "When the in-game squads were last synced. {date} is a formatted date.",
    },
    "squad.slots.gameUntitled": {
        text: "Squad {index}",
        description: "Name of an in-game squad that has no name. {index} is 1 to 4.",
    },
    "squad.slots.gameAria": {
        text: "Load a copy of in-game squad {name}",
        description: "Accessible name of an in-game squad card. {name} is the squad's name.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
