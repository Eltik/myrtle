import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "story";

export const messages = {
    "progress.finished": {
        text: "Finished",
        description: "Label of the stat showing how many stories are read.",
    },
    "progress.finishedValue": {
        text: "{read} of {total} stories",
        description: "Value of the finished-stories stat.",
    },
    "progress.share": {
        text: "Share read",
        description: "Label of the stat showing the percentage of stories read.",
    },
    "progress.chapters": {
        text: "Chapters finished",
        description: "Label of the stat counting fully-read chapters.",
    },
    "progress.chaptersValue": {
        text: "{done} of {total}",
        description: "Value of the chapters-finished stat.",
    },
    "progress.words": {
        text: "Words read",
        description: "Label of the stat summing the word counts of the stories read.",
    },
    "progress.wordsPending": {
        text: "Not counted yet",
        description: "Value of the words-read stat while the backend sends no word counts.",
    },
    "progress.manage": {
        text: "Your progress",
        description: "Heading above the backup, restore and reset buttons.",
    },
    "progress.manageBlurb": {
        text: "Reading progress lives in this browser only. Back it up before clearing site data, and restore it on another device.",
        description: "Explanation under the progress heading, shown to a signed-out reader.",
    },
    "progress.manageBlurbSynced": {
        text: "Reading progress lives in this browser and on your account, so it follows you to another device. Back it up before clearing site data.",
        description: "Explanation under the progress heading, shown to a signed-in reader whose progress syncs.",
    },
    "progress.sync": {
        text: "Sync with account",
        description: "Heading of the row that reports whether reading progress is synced to the account.",
    },
    "progress.sync.signedOut": {
        text: "Sign in to sync",
        description: "State line shown to a signed-out reader, above a button that opens the login dialog.",
    },
    "progress.sync.signIn": {
        text: "Sign in",
        description: "Button that opens the login dialog from the sync row.",
    },
    "progress.sync.pending": {
        text: "Not synced yet",
        description: "State line before the first sync of the session has finished.",
    },
    "progress.sync.syncing": {
        text: "Syncing",
        description: "State line while a sync is in flight.",
    },
    "progress.sync.synced": {
        text: "Synced {when}",
        description: 'State line after a successful sync. {when} is a relative time such as "5 minutes ago".',
    },
    "progress.sync.offline": {
        text: "Offline. Your progress is safe in this browser and syncs when you are back.",
        description: "State line when the sync could not reach the site.",
    },
    "progress.sync.unauthorized": {
        text: "Your session has expired. Sign in again to keep syncing.",
        description: "State line when the sync was refused because the session is no longer valid.",
    },
    "progress.sync.failed": {
        text: "Sync failed. Your progress is safe in this browser.",
        description: "State line when the sync failed for any other reason.",
    },
    "progress.sync.gameImported": {
        text: "{count} stories read in-game ({archived} in the Archive), imported. Game data synced {when}.",
        description:
            'Second line of the sync row: how many stories the Arknights client says were read, how many of those its own Archive lists, and when the account last refreshed its game data. {count} is the union of the client\'s played-script record and the Archive, so {archived} is always the smaller number. {when} is a relative time such as "5 minutes ago".',
    },
    "progress.sync.gameNone": {
        text: "No stories read in-game yet, as of your last game data refresh {when}.",
        description: "Second line of the sync row when the account has refreshed its game data but the game reported nothing read.",
    },
    "progress.sync.gameNever": {
        text: "Nothing imported from the game yet. Refresh your profile to bring in what you have read in-game.",
        description: "Second line of the sync row when the account has never refreshed its game data.",
    },
    "progress.sync.now": {
        text: "Sync now",
        description: "Button that starts a sync immediately.",
    },
    "progress.sync.retry": {
        text: "Retry",
        description: "Button that starts a sync again after one failed.",
    },
    "progress.backup": {
        text: "Back up",
        description: "Button that downloads the reading progress as a JSON file.",
    },
    "progress.restore": {
        text: "Restore",
        description: "Button that loads reading progress from a JSON file.",
    },
    "progress.reset": {
        text: "Reset",
        description: "Button that clears all reading progress.",
    },
    "progress.restored": {
        text: "Restored {stories} finished stories and {positions} saved positions.",
        description: "Confirmation after a progress file is loaded.",
    },
    "progress.restoreFailed": {
        text: "That file holds no reading progress.",
        description: "Error after loading a file that is not a progress backup.",
    },
    "progress.wasReset": {
        text: "Reading progress cleared.",
        description: "Confirmation after progress is reset.",
    },
    "progress.reset.title": {
        text: "Clear all reading progress?",
        description: "Title of the reset confirmation dialog.",
    },
    "progress.reset.body": {
        text: "Every finished story and saved position is removed from this browser. This cannot be undone.",
        description: "Body of the reset confirmation dialog, shown to a signed-out reader.",
    },
    "progress.reset.bodySynced": {
        text: "Every finished story and saved position is removed from this browser and from your account, on every device. This cannot be undone.",
        description: "Body of the reset confirmation dialog, shown to a signed-in reader whose progress syncs.",
    },
    "progress.reset.cancel": {
        text: "Cancel",
        description: "Button that dismisses the reset confirmation dialog.",
    },
    "progress.reset.confirm": {
        text: "Clear progress",
        description: "Button that confirms clearing reading progress.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
