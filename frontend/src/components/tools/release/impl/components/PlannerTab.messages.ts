import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tools";

export const messages = {
    "release.planner.showPast": {
        text: "Show past",
        description: "Switch label: also list events and sales that have already happened.",
    },
    "release.planner.hideOwned": {
        text: "Hide owned",
        description: "Switch label: drop outfits the signed-in account already owns from every event's outfit list.",
    },
    "release.planner.ownedHidden": {
        text: "{count, plural, one {# outfit you already own is hidden.} other {# outfits you already own are hidden.}}",
        description: "Note under an event's outfits when the hide-owned switch has removed some; also the whole outfit section when it removed all of them.",
    },
    "release.planner.blurb": {
        text: "First-clear Originite Prime included.",
        description: "Short note above the planner. 'First-clear' is the one-off reward for three-starring a stage; 'Originite Prime' is the game's premium currency.",
    },
    "release.planner.saving": {
        text: "Saving to your account…",
        description: "Status while the plan is being written to the signed-in account. Keep the single ellipsis character.",
    },
    "release.planner.saved": {
        text: "Last saved to your account: {date}",
        description: "Status after the plan was written to the signed-in account, naming when it was last saved.",
    },
    "release.planner.autosave": {
        text: "Saved to your account as you go.",
        description: "Status telling a signed-in reader the plan needs no saving.",
    },
    "release.planner.signedOut": {
        text: "Sign in to keep the plan on your account; until then it stays in this browser.",
        description: "Status telling a signed-out reader where the plan is kept.",
    },
    "release.planner.empty.title": {
        text: "Nothing ahead",
        description: "Empty-state heading when no dated event or sale is coming up.",
    },
    "release.planner.empty.desc": {
        text: "No upcoming EN event or store sale has a date. Turn on Show past to see recent ones.",
        description: "Empty-state body. 'EN' is the English game server; 'Show past' names the switch above, so keep it matching.",
    },
    "release.planner.summaryButton": {
        text: "Selection summary",
        description: "Button that switches the right pane to the totals across every event.",
    },
    "release.planner.reset": {
        text: "Reset the plan",
        description: "Accessible name of the button that clears every pick and stage tick.",
    },
    "release.planner.initial": {
        text: "Initial primes",
        description: "Label of the input holding how much Originite Prime the plan starts with. 'Primes' is short for Originite Prime.",
    },
    "release.planner.notSynced": {
        text: "not synced yet: ",
        description: "Lead-in before the link that pulls the balance from the player's account. Keep the trailing space.",
    },
    "release.planner.syncing": {
        text: "syncing…",
        description: "Label of the sync link while the request is in flight. Lowercase, mid-sentence; keep the single ellipsis character.",
    },
    "release.planner.syncNow": {
        text: "sync my account",
        description: "Link that pulls the Originite Prime balance from the player's account. Lowercase, mid-sentence.",
    },
    "release.planner.syncFailed": {
        text: " (failed)",
        description: "Appended after the sync link when the request failed. Keep the leading space and the parentheses.",
    },
    "release.planner.useAccount": {
        text: "use my account's {count}",
        description: "Link that replaces a hand-typed balance with the one from the player's account.",
    },
    "release.planner.fromAccount": {
        text: "from your account",
        description: "Note saying the balance came from the player's account. An optional 'synced' clause may follow.",
    },
    "release.planner.fromAccountSynced": {
        text: ", synced {date}",
        description: "Optional clause saying when the account balance was last pulled. Keep the leading comma and space.",
    },
    "release.planner.showSummary": {
        text: "Show summary",
        description: "Narrow-screen button that switches to the totals pane.",
    },
    "release.planner.showSelected": {
        text: "Show the selected event",
        description: "Narrow-screen button that switches back to the chosen event's detail.",
    },
    "release.planner.showEvents": {
        text: "Show events",
        description: "Narrow-screen button that switches back to the event list.",
    },
    "release.planner.tag.review": {
        text: "Fashion Review",
        description: "Tag on a Fashion Review row. The game's own name for the sale.",
    },
    "release.planner.tag.listing": {
        text: "Store sale",
        description: "Tag on a row grouping outfits that go on sale without a matching event.",
    },
    "release.planner.tag.rerun": {
        text: "Rerun",
        description: "Tag on an event that is being run again.",
    },
    "release.planner.tag.event": {
        text: "Event",
        description: "Tag on an event running for the first time.",
    },
    "release.planner.card.estimated": {
        text: " est.",
        description: "Appended after a date that is only an estimate. Abbreviation of 'estimated'; keep the leading space.",
    },
    "release.planner.card.outfits": {
        text: "{count, plural, one { · # outfit} other { · # outfits}}",
        description: "How many outfits arrive with a row, appended after its date. Keep the leading space and middle dot.",
    },
    "release.planner.card.picked": {
        text: ", {count} picked",
        description: "How many of a row's outfits are in the plan. Keep the leading comma and space.",
    },
    "release.planner.stages": {
        text: "Event stages",
        description: "Heading over the ticklist of stages that pay Originite Prime.",
    },
    "release.planner.stages.income": {
        text: "{income} of {potential}",
        description: "How much of a row's Originite Prime the plan counts on; the currency icon follows.",
    },
    "release.planner.stages.unknownTitle": {
        text: "{count, plural, one {# stage has} other {# stages have}} no record on your account; tick any you did not clear.",
        description: "Tooltip on the stage tally when some stages have no clear record.",
    },
    "release.planner.stages.claimed": {
        text: "{count} claimed",
        description: "Part of the stage tally: stages whose reward is already taken.",
    },
    "release.planner.stages.open": {
        text: "{count} open",
        description: "Part of the stage tally: stages whose reward is still available.",
    },
    "release.planner.stages.unrated": {
        text: "{count} cleared, rating not on record",
        description: "Part of the stage tally: stages cleared, but with no star rating recorded.",
    },
    "release.planner.stages.unknown": {
        text: "{count} no record",
        description: "Part of the stage tally: stages the account says nothing about.",
    },
    "release.planner.stages.tallyJoin": {
        text: " · ",
        description: "Separator between the parts of the stage tally. Keep the spaces around the middle dot.",
    },
    "release.planner.stages.resetAccount": {
        text: "Back to what your account shows",
        description: "Tooltip on the reset link when the player's account can supply the defaults.",
    },
    "release.planner.stages.resetDefaults": {
        text: "Back to the defaults",
        description: "Tooltip on the reset link when there is no account to fall back on.",
    },
    "release.planner.stages.resetAll": {
        text: "Reset all",
        description: "Link that puts every stage tick back to its default.",
    },
    "release.planner.stages.clearAll": {
        text: "Clear all",
        description: "Link that unticks every stage.",
    },
    "release.planner.stages.selectAll": {
        text: "Select all",
        description: "Link that ticks every stage.",
    },
    "release.planner.stages.cm": {
        text: " CM",
        description: "Appended to a Challenge Mode stage's code. 'CM' is the game's own abbreviation; keep the leading space.",
    },
    "release.planner.stages.multiplier": {
        text: " ×{count}",
        description: "Appended to a stage that pays more than one Originite Prime. Keep the leading space and the multiplication sign.",
    },
    "release.planner.status.claimed": {
        text: "Three-starred on your account: its Originite Prime is already claimed",
        description: "Tooltip on a stage the player has fully cleared.",
    },
    "release.planner.status.open": {
        text: "Your account shows this short of three stars: its Originite Prime is still available",
        description: "Tooltip on a stage whose reward is still to be had.",
    },
    "release.planner.status.unrated": {
        text: "Cleared on your account, star rating not on record",
        description: "Tooltip on a stage cleared with no star rating recorded.",
    },
    "release.planner.noOutfit": {
        text: "No outfit arrives with this one.",
        description: "Shown when an event brings no outfits at all.",
    },
    "release.planner.group.rerun": {
        text: "Rerun",
        description: "Tag on a group of outfits going back on sale.",
    },
    "release.planner.group.new": {
        text: "New",
        description: "Tag on a group of outfits arriving for the first time.",
    },
    "release.planner.summary.title": {
        text: "Selection summary",
        description: "Heading of the pane totalling every event in the plan.",
    },
    "release.planner.summary.start": {
        text: "start",
        description: "Label before the Originite Prime the plan begins with. Lowercase; it sits in a dense figure row.",
    },
    "release.planner.summary.end": {
        text: "end",
        description: "Label before the Originite Prime the plan finishes with. Lowercase; it sits in a dense figure row.",
    },
    "release.planner.summary.short": {
        text: "Short by {count} at {event} ({date}).",
        description: "Warning that the plan runs out of Originite Prime, naming where. {event} is an event name from the game data.",
    },
    "release.planner.summary.empty": {
        text: "No outfits picked yet. Open an event on the left and pick the ones you want.",
        description: "Shown in the summary pane while no outfit has been picked. Events without a picked outfit are left out of the summary.",
    },
    "release.planner.summary.remove": {
        text: "Remove",
        description: "Accessible name of the cross that drops one outfit from the plan.",
    },
} satisfies MessageMap;

// `dynamic`: the stage-status keys are stored in a lookup table and resolved as
// `t(STATUS_TITLE_KEYS[status])`, so the
// extractor has no literal call site to match them against.
export const { keys } = defineMessages({ namespace, messages, dynamic: true });
