/**
 * Hand-written release notes. This is the CURATED tier: the `/changelog` commit
 * feed answers "what did you touch", these answer "what should I go look at".
 *
 * Adding an entry with `announce: true` is what lights the bell in the header.
 * Nothing auto-opens; the visitor chooses when to read it. There is no build id
 * and no deploy hook: the newest announced `id` IS the cache key, so a lint
 * commit or an asset repack changes nothing.
 *
 * Ids are date-prefixed so they sort lexically. Add a `-2` suffix for a second
 * entry on the same day. Never reuse an id unless you want it re-shown.
 */

export type ReleaseNoteKind = "new" | "improved" | "fixed";

export interface IReleaseNoteItem {
    kind: ReleaseNoteKind;
    text: string;
}

export interface IReleaseNote {
    /** Sortable, date-prefixed. Stored verbatim in localStorage as the seen marker. */
    id: string;
    /** ISO date, rendered in the dialog eyebrow. */
    date: string;
    title: string;
    /** `false` files the entry in the archive without interrupting anyone. */
    announce: boolean;
    /** Markdown. One to three sentences: this is the marquee, not a bullet list. */
    lead: string;
    media?: { src: string; alt: string };
    /** Secondary changes, one line each. Deliberately subordinate to `lead`. */
    items?: IReleaseNoteItem[];
    /** Deep link to the thing that changed. Becomes the dialog's primary action. */
    href?: string;
    hrefLabel?: string;
}

const ENTRIES: IReleaseNote[] = [
    {
        id: "2026-09-10",
        date: "2026-09-10",
        title: "Build statistics",
        announce: true,
        lead: "Based on community statistics, operators now display what users build for masteries and modules. More bug fixes and grading have been improved.",
        href: "/operators",
        hrefLabel: "Browse operators",
        items: [
            { kind: "new", text: "Added changelog notifications." },
            { kind: "new", text: "Added build statistics." },
            { kind: "new", text: "Display breakpoints and percentages for mastery/module levels." },
            { kind: "improved", text: "Operator skills/modules default to what is most used." },
            { kind: "improved", text: "Operator grades were computed from the wrong baseline. Scores across the roster have shifted." },
            { kind: "fixed", text: "Profile rosters no longer play E2 dynamic art over an operator who has not reached E2." },
            { kind: "fixed", text: "Check-in dates and Reclamation Algorithm scoring." },
        ],
    },
];

/** Newest first. Sorted here so authoring order in `ENTRIES` never matters. */
export const RELEASE_NOTES: readonly IReleaseNote[] = [...ENTRIES].sort((a, b) => (a.id < b.id ? 1 : a.id > b.id ? -1 : 0));

/** The entry the dialog shows, or `null` when nothing is announceable. */
export const LATEST_ANNOUNCED_NOTE: IReleaseNote | null = RELEASE_NOTES.find((note) => note.announce) ?? null;

/**
 * Announced entries strictly newer than `seenId`, excluding the marquee itself.
 * Drives the "N earlier updates" footer line so we never stack dialogs.
 */
export function countMissedNotes(seenId: string | null): number {
    if (seenId === null || LATEST_ANNOUNCED_NOTE === null) return 0;
    return RELEASE_NOTES.filter((note) => note.announce && note.id > seenId && note.id !== LATEST_ANNOUNCED_NOTE.id).length;
}
