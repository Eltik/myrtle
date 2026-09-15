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
 *
 * The prose itself lives in `entries.messages.ts`: an entry carries message
 * KEYS and whichever component renders it resolves them with `t()`. A new entry
 * therefore needs its keys added there too.
 */

import type { messages as entryMessages } from "./entries.messages";

export type ReleaseNoteKind = "new" | "improved" | "fixed";

/** A key declared in `entries.messages.ts`. */
export type ReleaseNoteMessageKey = keyof typeof entryMessages & string;

export interface IReleaseNoteItem {
    kind: ReleaseNoteKind;
    textKey: ReleaseNoteMessageKey;
}

export interface IReleaseNote {
    /** Sortable, date-prefixed. Stored verbatim in localStorage as the seen marker. */
    id: string;
    /** ISO date, rendered in the dialog eyebrow. */
    date: string;
    titleKey: ReleaseNoteMessageKey;
    /** `false` files the entry in the archive without interrupting anyone. */
    announce: boolean;
    /** Markdown. One to three sentences: this is the marquee, not a bullet list. */
    leadKey: ReleaseNoteMessageKey;
    media?: { src: string; altKey: ReleaseNoteMessageKey };
    /** Secondary changes, one line each. Deliberately subordinate to `lead`. */
    items?: IReleaseNoteItem[];
    /** Deep link to the thing that changed. Becomes the dialog's primary action. */
    href?: string;
    hrefLabelKey?: ReleaseNoteMessageKey;
}

const ENTRIES: IReleaseNote[] = [
    {
        id: "2026-09-16",
        date: "2026-09-16",
        titleKey: "note.2026-09-16.title",
        announce: true,
        leadKey: "note.2026-09-16.lead",
        href: "/tools/release",
        hrefLabelKey: "note.2026-09-16.hrefLabel",
        items: [
            { kind: "new", textKey: "note.2026-09-16.item.1" },
            { kind: "new", textKey: "note.2026-09-16.item.2" },
            { kind: "improved", textKey: "note.2026-09-16.item.3" },
            { kind: "improved", textKey: "note.2026-09-16.item.4" },
            { kind: "fixed", textKey: "note.2026-09-16.item.5" },
        ],
    },
    {
        id: "2026-09-10",
        date: "2026-09-10",
        titleKey: "note.2026-09-10.title",
        announce: true,
        leadKey: "note.2026-09-10.lead",
        href: "/operators",
        hrefLabelKey: "note.2026-09-10.hrefLabel",
        items: [
            { kind: "new", textKey: "note.2026-09-10.item.1" },
            { kind: "new", textKey: "note.2026-09-10.item.2" },
            { kind: "new", textKey: "note.2026-09-10.item.3" },
            { kind: "improved", textKey: "note.2026-09-10.item.4" },
            { kind: "improved", textKey: "note.2026-09-10.item.5" },
            { kind: "fixed", textKey: "note.2026-09-10.item.6" },
            { kind: "fixed", textKey: "note.2026-09-10.item.7" },
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
