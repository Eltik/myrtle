import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

/**
 * Labels for the pending-change list the editor shows before a save.
 * `state.ts` has no React, so `diffStates` takes a `t` from the component
 * that renders its output.
 */
export const namespace = "tierLists";

export const messages = {
    "edit.change.listDetails": {
        text: "List details",
        description: "Pending change: the list's title or description was edited.",
    },
    "edit.change.tierCreated": {
        text: 'New tier "{name}"',
        description: "Pending change: a tier was added. {name} is the label the author gave it; keep the quotes.",
    },
    "edit.change.tierUpdated": {
        text: 'Updated "{name}"',
        description: "Pending change: a tier's label, colour or description was edited. {name} is the author's own label; keep the quotes.",
    },
    "edit.change.tierDeleted": {
        text: 'Deleted "{name}"',
        description: "Pending change: a tier was removed. {name} is the author's own label; keep the quotes.",
    },
    "edit.change.tiersReordered": {
        text: "Reordered tiers",
        description: "Pending change: the tiers were moved into a different order.",
    },
    "edit.change.placed": {
        text: "{count, plural, one {# operator} other {# operators}} placed",
        description: "Pending change: operators were added to a tier.",
    },
    "edit.change.moved": {
        text: "{count, plural, one {# operator} other {# operators}} moved",
        description: "Pending change: operators were moved from one tier to another.",
    },
    "edit.change.reordered": {
        text: "{count, plural, one {# operator} other {# operators}} reordered",
        description: "Pending change: operators changed position within their own tier.",
    },
    "edit.change.unplaced": {
        text: "{count, plural, one {# operator} other {# operators}} unplaced",
        description: "Pending change: operators were taken off the board and back into the pool.",
    },
    "edit.change.described": {
        text: "{count, plural, one {# description} other {# descriptions}} edited",
        description: "Pending change: the notes attached to placed operators were edited.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
