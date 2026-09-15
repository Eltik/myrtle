import { defineMessages, type MessageMap } from "#/lib/i18n/messages";

export const namespace = "admin";

export const messages = {
    "audit.kicker": {
        text: "Operate",
        description: "Eyebrow over the Audit log page title, naming the admin section it belongs to. Rendered uppercase.",
    },
    "audit.title": {
        text: "Audit log",
        description: "Title of the admin screen listing every recorded edit.",
    },
    "audit.sub": {
        text: "Append-only edit trail from {table}. Each row was written when an admin saved an operator note. Permission grants and tier-list publishes don't yet emit audit rows in v3.",
        description: "Sentence under the Audit log title. {table} is a database table name, shown in monospace. 'v3' is this version of the site.",
    },
    "audit.refresh": {
        text: "Refresh",
        description: "Button that re-fetches the audit rows.",
    },
    "audit.searchPlaceholder": {
        text: "Filter by actor, operator, field, content…",
        description: "Prompt in the audit search box. The 'actor' is whoever made the edit. Keep the ellipsis character.",
    },
    "audit.filter.all": {
        text: "All fields",
        description: "Filter option that shows edits to any note field.",
    },
    "audit.filter.pros": {
        text: "Pros",
        description: "Filter option for edits to a note's strengths field.",
    },
    "audit.filter.cons": {
        text: "Cons",
        description: "Filter option for edits to a note's weaknesses field.",
    },
    "audit.filter.notes": {
        text: "Notes",
        description: "Filter option for edits to a note's long-form guidance field.",
    },
    "audit.filter.trivia": {
        text: "Trivia",
        description: "Filter option for edits to a note's trivia field.",
    },
    "audit.filter.summary": {
        text: "Summary",
        description: "Filter option for edits to a note's one-line summary field.",
    },
    "audit.countShown": {
        text: "{filtered} of {shown} shown",
        description: "How many audit rows survive the current filter. Both numbers are already formatted.",
    },
    "audit.countTotal": {
        text: "/ {total} total",
        description: "Aside after the shown-count giving how many audit rows exist server-side. The number is already formatted. Keep the leading slash.",
    },
    "audit.loadError": {
        text: "Failed to load audit log: {message}",
        description: "Shown when the audit request failed; {message} is the error text the server or browser gave.",
    },
    "audit.unknownError": {
        text: "unknown error",
        description: "Stand-in error text when the failed request carried no message.",
    },
    "audit.empty": {
        text: "No audit rows yet.",
        description: "Empty state when nothing has ever been edited.",
    },
    "audit.noMatch": {
        text: "No rows match your filter.",
        description: "Empty state when audit rows exist but none survive the current filter.",
    },
    "audit.th.when": {
        text: "When",
        description: "Audit table column header for the time of the edit. Rendered uppercase.",
    },
    "audit.th.actor": {
        text: "Actor",
        description: "Audit table column header for who made the edit. Rendered uppercase.",
    },
    "audit.th.operator": {
        text: "Operator",
        description: "Audit table column header for which operator's note was edited. 'Operator' is the game's word for a playable character. Rendered uppercase.",
    },
    "audit.th.field": {
        text: "Field",
        description: "Audit table column header for which note field changed. Rendered uppercase.",
    },
    "audit.th.change": {
        text: "Change",
        description: "Audit table column header for the before-and-after text. Rendered uppercase.",
    },
    "audit.actor.deletedTitle": {
        text: "Internal user_id {id} - referenced user no longer exists.",
        description: "Tooltip on an audit row whose author has been deleted. 'user_id' is the database column name and stays as-is.",
    },
    "audit.actor.deleted": {
        text: "Deleted user",
        description: "Shown in place of a nickname when the account that made the edit no longer exists.",
    },
    "audit.uid": {
        text: "UID {uid}",
        description: "The editor's in-game account number beside their nickname. 'UID' is the game's own abbreviation and stays as-is.",
    },
    "audit.diff.added": {
        text: "added",
        description: "Badge on an audit row where a previously empty field was filled in.",
    },
    "audit.diff.cleared": {
        text: "cleared",
        description: "Badge on an audit row where a field was emptied.",
    },
} satisfies MessageMap;

export const { keys } = defineMessages({ namespace, messages });
