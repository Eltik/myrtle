import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "tools";

export const messages = {
    "release.override.kind.activity": {
        text: "Event",
        description: "Kind of a release-date override that moves a story event. Shown in the admin override editor and list.",
    },
    "release.override.kind.pool": {
        text: "Banner",
        description: "Kind of a release-date override that moves a gacha banner.",
    },
    "release.override.kind.skin": {
        text: "Outfit",
        description: "Kind of a release-date override that moves one outfit (character skin) sale.",
    },
    "release.override.kind.review": {
        text: "Fashion Review",
        description: "Kind of a release-date override that moves a Fashion Review, the game's recurring outfit rerun sale. Keep the game's own name for it.",
    },
    "release.override.reviewItem": {
        text: "Fashion Review, CN {date}",
        description: "Names a Fashion Review by the date its Chinese edition started. {date} is an already formatted date. 'CN' is the Chinese game server and stays as-is.",
    },
    "release.override.title.add": {
        text: "Override release date",
        description: "Title of the admin side sheet that sets an announced EN date for an event or banner by hand.",
    },
    "release.override.title.edit": {
        text: "Edit override",
        description: "Title of the same side sheet when the row already carries a hand-set date.",
    },
    "release.override.subtitle": {
        text: "{name} · {kind}",
        description: "Subtitle under the sheet title: the item's name, then its kind (e.g. 'Event'). The CN id follows it in a code font.",
    },
    "release.override.current.estimated": {
        text: "Currently estimated for {date} ({lo} to {hi}).",
        description: "What the planner shows for this row today. {date} is the estimated EN date; {lo} and {hi} bound the window it could fall in. All three are formatted dates.",
    },
    "release.override.current.override": {
        text: "Currently announced for {date}, per {source}.",
        description: "The row already shows a hand-set date. {date} is that date or date range, formatted; {source} is where it came from, e.g. 'manual' or 'the EN stream'.",
    },
    "release.override.current.confirmed": {
        text: "Confirmed by EN gamedata for {date}.",
        description: "The English game data already carries this row. {date} is that date or date range, formatted. 'EN' is the English game server.",
    },
    "release.override.current.confirmedNote": {
        text: "EN gamedata already confirms this date; an override is ignored while the sighting exists.",
        description: "Warning under the confirmed line: a date found in the English game data always wins over a hand-set one, so saving here changes nothing the reader sees.",
    },
    "release.override.current.none": {
        text: "No EN date is shown for this row yet.",
        description: "The row has no estimate and no announced date today. 'EN' is the English game server.",
    },
    "release.override.replacesExisting": {
        text: "An override already exists for this item. Saving replaces it.",
        description: "Hint in the 'new override' form when the kind and CN id typed match a row that already has a hand-set date.",
    },
    "release.override.field.kind": {
        text: "Kind",
        description: "Label of the select choosing what the override moves: an event, a banner, an outfit or a Fashion Review.",
    },
    "release.override.field.cnId": {
        text: "CN id",
        description: "Label of the field holding the Chinese server's id for the item. 'CN' is the Chinese game server.",
    },
    "release.override.field.cnIdHint.activity": {
        text: "The activity id, e.g. act42sre.",
        description: "Hint under the CN id field for an event override. 'act42sre' is a literal id example; keep it as-is.",
    },
    "release.override.field.cnIdHint.pool": {
        text: "The gacha pool id.",
        description: "Hint under the CN id field for a banner override.",
    },
    "release.override.field.cnIdHint.skin": {
        text: "The skin id.",
        description: "Hint under the CN id field for an outfit override.",
    },
    "release.override.field.cnIdHint.review": {
        text: "When the CN edition started, in unix seconds.",
        description: "Hint under the CN id field for a Fashion Review override: the id is the Chinese edition's start time as a unix timestamp. 'CN' is the Chinese game server.",
    },
    "release.override.field.enStart": {
        text: "EN start (UTC)",
        description: "Label of the date-and-time field for when the item opens on the English server, entered in UTC. 'EN' is the English game server.",
    },
    "release.override.field.enStartHint": {
        text: "When it opens on EN, in UTC.",
        description: "Hint under the EN start field.",
    },
    "release.override.field.enEnd": {
        text: "EN end (UTC, optional)",
        description: "Label of the date-and-time field for when the item closes on the English server, in UTC. It may be left empty.",
    },
    "release.override.field.enEndHint": {
        text: "Leave empty if the close date is not announced.",
        description: "Hint under the EN end field.",
    },
    "release.override.field.enId": {
        text: "EN id (optional)",
        description: "Label of the field for the English server's own id for the item, when it differs or is known.",
    },
    "release.override.field.enIdHint": {
        text: "The id EN uses, if known.",
        description: "Hint under the EN id field. 'EN' is the English game server.",
    },
    "release.override.field.enName": {
        text: "EN name (optional)",
        description: "Label of the field for the item's announced English name.",
    },
    "release.override.field.enNameHint": {
        text: "The announced English name, if any.",
        description: "Hint under the EN name field.",
    },
    "release.override.field.source": {
        text: "Source (optional)",
        description: "Label of the field naming where the hand-set date comes from.",
    },
    "release.override.field.sourceHint": {
        text: "Where the date comes from, e.g. an EN announcement.",
        description: "Hint under the Source field. Left empty, the source is saved as 'manual'.",
    },
    "release.override.field.note": {
        text: "Note (optional)",
        description: "Label of a free-text note kept with the override and shown in the date's tooltip.",
    },
    "release.override.err.cnIdRequired": {
        text: "Enter the CN id.",
        description: "Validation error: the CN id field is empty.",
    },
    "release.override.err.startRequired": {
        text: "Enter the EN start.",
        description: "Validation error: the EN start date is empty.",
    },
    "release.override.err.startInvalid": {
        text: "Enter a full date and time.",
        description: "Validation error: the EN start is not a complete, real date and time.",
    },
    "release.override.err.endInvalid": {
        text: "Enter a full date and time, or leave it empty.",
        description: "Validation error: the EN end is not a complete, real date and time.",
    },
    "release.override.err.endBeforeStart": {
        text: "The end must come after the start.",
        description: "Validation error: the EN end is at or before the EN start.",
    },
    "release.override.cancel": {
        text: "Cancel",
        description: "Button closing the override sheet or the remove confirmation without saving.",
    },
    "release.override.save": {
        text: "Save",
        description: "Submit button of the override sheet.",
    },
    "release.override.remove": {
        text: "Remove override",
        description: "Destructive button dropping the hand-set date from the row.",
    },
    "release.override.remove.title": {
        text: "Remove this override?",
        description: "Title of the confirmation before an override is removed.",
    },
    "release.override.remove.body": {
        text: "The row goes back to its estimate / gamedata date.",
        description: "Body of the remove confirmation: without the hand-set date the row shows the projected date, or the date in the English game data if there is one.",
    },
    "release.override.toast.saved": {
        text: "Override saved",
        description: "Toast title after an override is saved.",
    },
    "release.override.toast.savedDesc": {
        text: "{name} now opens on EN {date}.",
        description: "Toast body after saving. {name} is the item's name or id; {date} is the saved EN date, formatted.",
    },
    "release.override.toast.saveFailed": {
        text: "Couldn't save the override",
        description: "Toast title when saving an override fails; the reason follows in the body.",
    },
    "release.override.toast.removed": {
        text: "Override removed",
        description: "Toast title after an override is removed.",
    },
    "release.override.toast.removedDesc": {
        text: "{name} is back on its estimate / gamedata date.",
        description: "Toast body after removing. {name} is the item's name or id.",
    },
    "release.override.toast.removeFailed": {
        text: "Couldn't remove the override",
        description: "Toast title when removing an override fails; the reason follows in the body.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
