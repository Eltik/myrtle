import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "settings";

export const messages = {
    "data.sync.title": {
        text: "Sync & refresh",
        description: "Card title over the re-sync control. The ampersand is literal.",
    },
    "data.sync.desc": {
        text: "Pull the latest from Yostar. Re-sync any time; we replace your stored snapshot.",
        description: "Card description under the Sync & refresh title. 'Yostar' is the game's publisher.",
    },
    "data.sync.rowTitle": {
        text: "Re-sync game data",
        description: "Label of the row holding the re-sync button.",
    },
    "data.sync.paused": {
        text: "Syncing is paused while the game is down for maintenance. Back in {time}.",
        description: "Caption under the re-sync button while the game server is down for maintenance. {time} is a short span like '42m' until the announced end.",
    },
    "data.sync.pausedNoEnd": {
        text: "Syncing is paused while the game is down for maintenance.",
        description: "Caption under the re-sync button while the game server is down for maintenance and its end has passed or was never announced.",
    },
    "data.sync.rowDesc": {
        text: "Operators, stage progress, IS/RA, base, medals, inventory.",
        description: "Caption listing what a re-sync fetches. 'IS' is Integrated Strategies, 'RA' is Reclamation Algorithm and 'base' the player's home installation - all in-game systems.",
    },
    "data.yourData.title": {
        text: "Your data on Myrtle",
        description: "Card title over the export and profile links. 'Myrtle' is this site's name.",
    },
    "data.yourData.desc": {
        text: "Everything we store is visible on your profile page. For a portable copy or a GDPR request, email privacy@myrtle.moe.",
        description: "Card description under Your data on Myrtle. GDPR is the EU data-protection regulation.",
    },
    "data.viewProfile.title": {
        text: "View on your profile",
        description: "Label of the row linking to the visitor's own profile page.",
    },
    "data.viewProfile.desc": {
        text: "Roster, scores, stage progress, gacha history (if stored), and medals. The data we have on you is what shows up here.",
        description: "Caption under the view-on-profile row. 'Roster' is the set of owned operators; 'gacha' is the random-draw system.",
    },
    "data.viewProfile.action": {
        text: "Open my profile",
        description: "Button that opens the visitor's own profile page.",
    },
    "data.export.title": {
        text: "Request data export",
        description: "Label of the row about getting a portable copy of the stored data.",
    },
    "data.export.desc": {
        text: "Email privacy@myrtle.moe to receive a portable copy of your data.",
        description: "Caption under the data-export row. The address is literal and must not be changed.",
    },
    "data.export.action": {
        text: "Email privacy@myrtle.moe",
        description: "Button that opens a pre-addressed email. The address is literal and must not be changed.",
    },
    "data.linked.title": {
        text: "Linked Yostar account",
        description: "Card title over the connected game account's status and controls.",
    },
    "data.linked.desc": {
        text: "We never store your password or sign-in code. If you checked Keep me synced when you signed in, we keep an encrypted game token that lets us re-sync on your behalf. Disconnect below to delete it.",
        description: "Card description under Linked Yostar account. It must stay true: the password and code are never stored; the token is stored only when the Keep me synced box in the sign-in dialog was checked.",
    },
    "data.signOut.title": {
        text: "Sign out of this browser",
        description: "Label of the row that ends the current session.",
    },
    "data.signOut.desc": {
        text: "Ends the current session. Your data stays; sign back in to see it again.",
        description: "Caption under the sign-out row.",
    },
    "data.signOut.action": {
        text: "Sign out",
        description: "Button that ends the current browser session.",
    },
    "data.disconnect.title": {
        text: "Disconnect game account",
        description: "Label of the row that deletes the stored game token.",
    },
    "data.disconnect.desc": {
        text: "Deletes the stored Yostar token, so we can no longer reach your account. Your synced data stays on your profile; the next re-sync will ask for a new email code.",
        description: "Caption under the disconnect row.",
    },
    "data.disconnect.action": {
        text: "Disconnect",
        description: "Button that deletes the stored game token.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
