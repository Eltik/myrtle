import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "settings";

export const messages = {
    "nav.account": {
        text: "Account",
        description: "Settings section for the visitor's game-synced account details, syncing, exporting and disconnecting. Replaces the former separate Profile and Account & data sections.",
    },
    "nav.appearance": {
        text: "Appearance",
        description: "Settings section for theme, accent colour and artwork options.",
    },
    "nav.privacy": {
        text: "Privacy",
        description: "Settings section for profile visibility and leaderboard opt-in.",
    },
    "nav.danger": {
        text: "Danger zone",
        description: "Settings section holding account deletion. Deliberately alarming.",
    },
    "toast.saved.title": {
        text: "Settings saved",
        description: "Success toast title after a privacy switch is stored.",
    },
    "toast.saved.body": {
        text: "Your privacy preferences are up to date.",
        description: "Success toast body after a privacy switch is stored.",
    },
    "toast.saveFailed.title": {
        text: "Couldn't save settings",
        description: "Error toast title when storing a privacy switch failed. The body is the server's own message.",
    },
    "toast.resynced.title": {
        text: "Roster re-synced",
        description: "Success toast title after pulling fresh game data. 'Roster' is the visitor's set of owned operators.",
    },
    "toast.resynced.body": {
        text: "Pulled the latest snapshot from Yostar.",
        description: "Success toast body after pulling fresh game data. 'Yostar' is the game's publisher.",
    },
    "toast.resyncFailed.title": {
        text: "Couldn't re-sync",
        description: "Error toast title when pulling fresh game data failed. The body is the server's own message.",
    },
    "toast.disconnected.title": {
        text: "Game account disconnected",
        description: "Success toast title after the stored game token was deleted.",
    },
    "toast.disconnected.body": {
        text: "We deleted the stored Yostar token. Your synced data is untouched; re-syncing will ask for a new email code.",
        description: "Success toast body after the stored game token was deleted.",
    },
    "toast.nothingToDisconnect.title": {
        text: "Nothing stored to disconnect",
        description: "Toast title when disconnect ran but no game token was held.",
    },
    "toast.nothingToDisconnect.body": {
        text: "We were not holding a Yostar token for your account.",
        description: "Toast body when disconnect ran but no game token was held.",
    },
    "toast.disconnectFailed.title": {
        text: "Couldn't disconnect",
        description: "Error toast title when deleting the stored game token failed. The body is the server's own message.",
    },
    "toast.signOutFailed.title": {
        text: "Couldn't sign out",
        description: "Error toast title when ending the browser session failed. The body is the server's own message.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
