import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tools";

export const messages = {
    "release.overrides.open": {
        text: "Overrides",
        description: "Admin button in the Release Planner header opening the list of hand-set release dates. A count badge follows it.",
    },
    "release.overrides.openLabel": {
        text: "Overrides ({count})",
        description: "Accessible name of the same button, with the number of stored overrides. {count} is a number.",
    },
    "release.overrides.title": {
        text: "Release date overrides",
        description: "Title of the admin sheet listing every hand-set EN release date.",
    },
    "release.overrides.description": {
        text: "Hand-set EN dates. A date in the EN gamedata still wins over any of these.",
        description: "Subtitle of the override list. 'EN' is the English game server; 'gamedata' is the game's own data files.",
    },
    "release.overrides.add": {
        text: "Add override",
        description: "Button opening an empty override form where the editor picks the kind and CN id.",
    },
    "release.overrides.col.kind": {
        text: "Kind",
        description: "Column header: event, banner, outfit or Fashion Review.",
    },
    "release.overrides.col.item": {
        text: "Item",
        description: "Column header: the event or banner the override moves.",
    },
    "release.overrides.col.start": {
        text: "EN start",
        description: "Column header: the hand-set English server start date. 'EN' is the English game server.",
    },
    "release.overrides.col.end": {
        text: "EN end",
        description: "Column header: the hand-set English server end date.",
    },
    "release.overrides.col.source": {
        text: "Source",
        description: "Column header: where the date comes from.",
    },
    "release.overrides.col.note": {
        text: "Note",
        description: "Column header: the free-text note kept with the override.",
    },
    "release.overrides.col.updated": {
        text: "Updated",
        description: "Column header: when the override was last saved.",
    },
    "release.overrides.col.actions": {
        text: "Actions",
        description: "Visually hidden column header over each row's edit and remove buttons.",
    },
    "release.overrides.none": {
        text: "None",
        description: "Placeholder in the EN start or EN end column when that date was not set.",
    },
    "release.overrides.edit": {
        text: "Edit override for {name}",
        description: "Accessible name of a row's edit button. {name} is the item's name or id.",
    },
    "release.overrides.remove": {
        text: "Remove override for {name}",
        description: "Accessible name of a row's remove button. {name} is the item's name or id.",
    },
    "release.overrides.empty.title": {
        text: "No overrides",
        description: "Empty-state title when no hand-set dates are stored.",
    },
    "release.overrides.empty.description": {
        text: "Every row shows its estimate or its EN gamedata date. Add one when EN announces a date early.",
        description: "Empty-state body. 'EN' is the English game server.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
