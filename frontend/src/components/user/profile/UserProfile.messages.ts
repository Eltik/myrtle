import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "user";

export const messages = {
    "profile.tab.showcase": {
        text: "Showcase",
        description: "Profile tab: the blocks the player chose to show first (favourite operators, skins and more, grids, tier lists, plans).",
    },
    "profile.tab.stats": {
        text: "Stats",
        description: "Profile tab: collection and account statistics.",
    },
    "profile.tab.score": {
        text: "Score",
        description: "Profile tab: the account's graded score breakdown.",
    },
    "profile.tab.roster": {
        text: "Roster",
        description: "Profile tab: the operators this account owns.",
    },
    "profile.tab.plans": {
        text: "Plans",
        description: "Profile tab: the account's public upgrade plans.",
    },
    "profile.tab.inventory": {
        text: "Inventory",
        description: "Profile tab: the items this account holds.",
    },
    "profile.tab.enemies": {
        text: "Enemies",
        description: "Profile tab: which enemies the account has encountered.",
    },
    "profile.tab.optimizer": {
        text: "Optimizer",
        description: "Profile tab: tools that suggest improvements to the account.",
    },
    "profile.tab.pulls": {
        text: "Pulls",
        description: "Profile tab: the account's gacha pull history, shown to visitors only when the player made it public. A pull is one draw on a headhunting banner.",
    },
    "profile.tab.lookback": {
        text: "Lookback",
        description: "Profile tab: snapshots of how built the account was at each event launch, and how it changed between them.",
    },
    "profile.notFound.eyebrow": {
        text: "Player",
        description: "Small uppercase label above the 'profile not found' heading. 'Player' is the site's term; the game itself says 'Doctor'.",
    },
    "profile.notFound.title": {
        text: "Profile not found",
        description: "Heading shown when the requested profile does not exist or is private.",
    },
    "profile.notFound.desc": {
        text: "No player with ID {id} exists, or their profile is private.",
        description: "Body of the 'profile not found' page. {id} is the account ID, rendered in monospace, and may move wherever the sentence needs it. 'Player' is the site's term; the game itself says 'Doctor'.",
    },
    "profile.noTabs.eyebrow": {
        text: "Private",
        description: "Small uppercase label above the note shown when a player hid every tab of their profile.",
    },
    "profile.noTabs.desc": {
        text: "This player keeps every section of their profile private.",
        description: "Shown under another player's profile header when they hid every tab, so there is nothing else to view.",
    },
    "profile.loadFailed.profile": {
        text: "This profile could not be loaded. The site may be having trouble; try again in a moment.",
        description: "Shown in place of a profile page when loading it failed for a reason other than the profile not existing or being private (a network error or server outage). A retry button follows.",
    },
    "profile.loadFailed.roster": {
        text: "This player's operators could not be loaded.",
        description: "Shown in place of a profile tab that needs the player's operator list (Stats, Roster, Plans, Optimizer) when loading that list failed. A retry button follows.",
    },
    "profile.loadFailed.inventory": {
        text: "This player's items could not be loaded.",
        description: "Shown in place of the profile's Inventory tab when loading the items failed. A retry button follows.",
    },
    "profile.loadFailed.score": {
        text: "This player's score could not be loaded.",
        description: "Shown in place of the profile's Score tab when loading the score failed. A retry button follows.",
    },
    "profile.loadFailed.improvements": {
        text: "Upgrade suggestions could not be loaded, so the panels below show none.",
        description: "Shown above the Score tab's cards when the suggestions of what to upgrade next failed to load; the score itself loaded. A retry button follows.",
    },
    "profile.loadFailed.enemies": {
        text: "This player's encountered enemies could not be loaded.",
        description: "Shown in place of the profile's Enemies tab when loading which enemies the player has met failed. A retry button follows.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
