import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "user";

export const messages = {
    "lookback.private": {
        text: "This player keeps their roster history private.",
        description: "Shown on the Lookback tab when the player hid their Roster tab, whose data the snapshots are.",
    },
    "lookback.loading": {
        text: "Loading snapshots…",
        description: "Placeholder while the account's snapshots load. Keep the single-character ellipsis.",
    },
    "lookback.timeline.title": {
        text: "Over time",
        description: "Card title over the chart of the account's counts (operators, Elite 2 and so on) across its snapshots.",
    },
    "lookback.metric.operators": {
        text: "Operators",
        description: "Chart metric chip and delta tile: operators owned.",
    },
    "lookback.metric.elite2": {
        text: "Elite 2",
        description: "Chart metric chip and delta tile: operators promoted to Elite 2.",
    },
    "lookback.metric.max_level": {
        text: "Max level",
        description: "Chart metric chip and delta tile: operators at the level cap of their current promotion.",
    },
    "lookback.metric.mastery3": {
        text: "Mastery 3",
        description: "Chart metric chip and delta tile: skills at Mastery 3 (counted per skill, not per operator).",
    },
    "lookback.metric.modules": {
        text: "Modules",
        description: "Chart metric chip and delta tile: unlocked modules.",
    },
    "lookback.metric.skins": {
        text: "Skins",
        description: "Chart metric chip and delta tile: non-default outfits owned.",
    },
    "lookback.series.recorded": {
        text: "Snapshots",
        description: "Legend entry for the solid line: counts recorded at each snapshot.",
    },
    "lookback.series.reconstructed": {
        text: "Owned, rebuilt from obtain dates",
        description: "Legend entry for the dashed line: operators owned over time, worked out from the date each operator was obtained.",
    },
    "lookback.series.now": {
        text: "Now",
        description: "Tooltip label of the chart's last point: the roster as of its latest sync.",
    },
    "lookback.reconstructed.note": {
        text: "The dashed line reaches back before snapshots began: it is rebuilt from the date each operator was obtained. It shows ownership only, since builds before the first snapshot were never recorded.",
        description: "Note under the chart explaining the reconstructed ownership line.",
    },
    "lookback.empty": {
        text: "No snapshot yet. One is taken on the first sync after each event launch.",
        description: "Shown when the account has no snapshot yet. A snapshot is recorded automatically when the account syncs.",
    },
    "lookback.compare.title": {
        text: "Compare",
        description: "Card title over the comparison of two snapshots.",
    },
    "lookback.compare.from": {
        text: "From",
        description: "Label of the picker for the earlier snapshot of the comparison.",
    },
    "lookback.compare.to": {
        text: "To",
        description: "Label of the picker for the later snapshot of the comparison.",
    },
    "lookback.compare.now": {
        text: "Now (last sync)",
        description: "Picker entry for the account as of its latest sync rather than a stored snapshot.",
    },
    "lookback.compare.week": {
        text: "Week of {date}",
        description: "Picker entry for a snapshot that no event launch opened: one per week. {date} is the week's Monday.",
    },
    "lookback.compare.event": {
        text: "{event} ({date})",
        description: "Picker entry for a snapshot opened by an event launch. {event} is the event's name, {date} the launch date.",
    },
    "lookback.compare.grade": {
        text: "Grade {from} → {to}",
        description: "Chip comparing the account grade letter at the two snapshots.",
    },
    "lookback.diff.gained": {
        text: "Operators gained",
        description: "Heading of the list of operators obtained between the two snapshots.",
    },
    "lookback.diff.elite2": {
        text: "New Elite 2",
        description: "Heading of the list of operators promoted to Elite 2 between the two snapshots.",
    },
    "lookback.diff.masteries": {
        text: "Masteries",
        description: "Heading of the list of skill masteries raised between the two snapshots.",
    },
    "lookback.diff.modules": {
        text: "Modules",
        description: "Heading of the list of modules unlocked or upgraded between the two snapshots.",
    },
    "lookback.diff.none": {
        text: "Nothing new",
        description: "Shown under a diff heading when nothing in that category changed.",
    },
    "lookback.diff.mastery": {
        text: "S{skill} M{from} → M{to}",
        description: "One mastery change. {skill} is the 1-based skill number, {from} and {to} the mastery ranks (0 to 3).",
    },
    "lookback.diff.module": {
        text: "Stage {from} → {to}",
        description: "One module upgrade between two stages (1 to 3).",
    },
    "lookback.diff.moduleNew": {
        text: "Unlocked, stage {to}",
        description: "One module unlocked between the snapshots, now at stage {to} (1 to 3).",
    },
    "lookback.diff.same": {
        text: "Pick two different points to compare.",
        description: "Shown when both pickers name the same snapshot.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
