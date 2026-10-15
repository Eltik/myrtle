import type { AdminNotesStatus } from "#/components/admin/shell/search";
import type { IOperatorNote } from "#/lib/api/operator-notes";
import { normalizeForSearch } from "#/lib/search/fuzzy";

/*
 * Pure logic behind the Operator notes section: the editable draft, the list
 * filters and the Save & next queue. No React, no auth, so it is unit-tested
 * next to this file.
 */

/** Same limits `updateOperatorNoteFn` enforces server-side. */
export const SUMMARY_LIMIT = 280;
export const TAG_LENGTH_LIMIT = 32;
export const TAG_COUNT_LIMIT = 16;

export const NOTE_FIELDS = ["summary", "pros", "cons", "notes", "trivia", "tags"] as const;
export type NoteField = (typeof NOTE_FIELDS)[number];

export interface INoteDraft {
    summary: string;
    pros: string;
    cons: string;
    notes: string;
    trivia: string;
    tags: string[];
}

export function draftFromNote(note: IOperatorNote | null | undefined): INoteDraft {
    return {
        summary: note?.summary ?? "",
        pros: note?.pros ?? "",
        cons: note?.cons ?? "",
        notes: note?.notes ?? "",
        trivia: note?.trivia ?? "",
        tags: note?.tags ?? [],
    };
}

export function draftsEqual(a: INoteDraft, b: INoteDraft): boolean {
    return a.summary === b.summary && a.pros === b.pros && a.cons === b.cons && a.notes === b.notes && a.trivia === b.trivia && a.tags.length === b.tags.length && a.tags.every((t, i) => t === b.tags[i]);
}

/** The five text fields a note is complete with. Tags are metadata and never count. */
export const TEXT_FIELDS = ["summary", "pros", "cons", "notes", "trivia"] as const;
export type TextField = (typeof TEXT_FIELDS)[number];

/** The list status of a note; the URL's `status` minus `all`. */
export type NoteStatus = Exclude<AdminNotesStatus, "all">;

/**
 * Whether anything is written. Mirrors `noteHasContent` (any field or tag),
 * which is what the admin bar's "missing notes" badge counts, so that badge
 * and the Empty chip agree.
 */
export function draftHasContent(draft: INoteDraft): boolean {
    return TEXT_FIELDS.some((field) => draft[field].trim().length > 0) || draft.tags.length > 0;
}

/** The text fields still blank (whitespace counts as blank), in form order. */
export function missingFields(draft: INoteDraft): TextField[] {
    return TEXT_FIELDS.filter((field) => draft[field].trim().length === 0);
}

/** Empty: nothing written. Incomplete: something written, not all five fields. Filled (Done): all five. */
export function draftStatus(draft: INoteDraft): NoteStatus {
    if (!draftHasContent(draft)) return "empty";
    return missingFields(draft).length > 0 ? "incomplete" : "filled";
}

/**
 * Adds a tag the way the tag input does on Enter: trimmed, cut to 32
 * characters, ignored when blank, already present or the note has 16.
 * Returns the same array when nothing changes.
 */
export function addTag(tags: readonly string[], raw: string): string[] {
    const tag = raw.trim().slice(0, TAG_LENGTH_LIMIT).trim();
    if (!tag || tags.includes(tag) || tags.length >= TAG_COUNT_LIMIT) return tags as string[];
    return [...tags, tag];
}

export interface INoteRow {
    id: string;
    name: string;
    rarity: number;
    note: IOperatorNote | null;
    status: NoteStatus;
    tags: readonly string[];
}

export interface IOperatorLike {
    id: string;
    name: string;
    rarity: number;
}

/** Every operator in the index, each with its note (if any): a LEFT JOIN of index and notes. */
export function joinRows(operators: readonly IOperatorLike[], notes: readonly IOperatorNote[]): INoteRow[] {
    const byOperator = new Map(notes.map((n) => [n.operator_id, n]));
    return operators.map((op) => {
        const note = byOperator.get(op.id) ?? null;
        return { id: op.id, name: op.name, rarity: op.rarity, note, status: draftStatus(draftFromNote(note)), tags: note?.tags ?? [] };
    });
}

export const NOTE_SORTS = ["name", "rarity", "recent"] as const;
export type NoteSort = (typeof NOTE_SORTS)[number];

export interface INoteFilters {
    status: AdminNotesStatus;
    q: string;
    rarities: readonly number[];
    /** OR match: a row passes when it carries any of these. */
    tags: readonly string[];
    sort: NoteSort;
}

export const DEFAULT_SORT: NoteSort = "name";

function updatedAt(row: INoteRow): number {
    const t = row.note ? Date.parse(row.note.updated_at) : Number.NaN;
    return Number.isFinite(t) ? t : Number.NEGATIVE_INFINITY;
}

export function sortRows(rows: readonly INoteRow[], sort: NoteSort, compareNames: (a: string, b: string) => number): INoteRow[] {
    const byName = (a: INoteRow, b: INoteRow) => compareNames(a.name, b.name) || a.id.localeCompare(b.id);
    const sorted = [...rows];
    if (sort === "rarity") sorted.sort((a, b) => b.rarity - a.rarity || byName(a, b));
    else if (sort === "recent") sorted.sort((a, b) => updatedAt(b) - updatedAt(a) || byName(a, b));
    else sorted.sort(byName);
    return sorted;
}

/** Search matches name, id or any tag, through the shared search normalization. */
function rowMatchesQuery(row: INoteRow, q: string): boolean {
    const needle = normalizeForSearch(q.trim());
    if (!needle) return true;
    return normalizeForSearch(row.name).includes(needle) || normalizeForSearch(row.id).includes(needle) || row.tags.some((t) => normalizeForSearch(t).includes(needle));
}

export function filterRows(rows: readonly INoteRow[], filters: INoteFilters, compareNames: (a: string, b: string) => number): INoteRow[] {
    const kept = rows.filter((row) => {
        if (filters.status !== "all" && row.status !== filters.status) return false;
        if (filters.rarities.length > 0 && !filters.rarities.includes(row.rarity)) return false;
        if (filters.tags.length > 0 && !filters.tags.some((t) => row.tags.includes(t))) return false;
        return rowMatchesQuery(row, filters.q);
    });
    return sortRows(kept, filters.sort, compareNames);
}

export function statusCounts(rows: readonly INoteRow[]): Record<AdminNotesStatus, number> {
    const counts = { all: rows.length, empty: 0, incomplete: 0, filled: 0 };
    for (const row of rows) counts[row.status]++;
    return counts;
}

/** Every tag in use across the roster, deduplicated and sorted. */
export function tagsInUse(rows: readonly INoteRow[], compare: (a: string, b: string) => number): string[] {
    const set = new Set<string>();
    for (const row of rows) for (const t of row.tags) set.add(t);
    return [...set].sort(compare);
}

/** The filter-button badge: panel filters only (rarities, tags, a non-default sort). */
export function activeFilterCount(filters: Pick<INoteFilters, "rarities" | "tags" | "sort">): number {
    return filters.rarities.length + filters.tags.length + (filters.sort !== DEFAULT_SORT ? 1 : 0);
}

export function toggle<T>(list: readonly T[], value: T): T[] {
    return list.includes(value) ? list.filter((v) => v !== value) : [...list, value];
}

/**
 * The operator Save & next opens: the first one after `currentId` in `order`
 * that isn't done (empty or incomplete), wrapping round, never `currentId`
 * itself. Null when none is left.
 */
export function nextUnfinishedOperator(order: readonly INoteRow[], currentId: string): INoteRow | null {
    const at = order.findIndex((r) => r.id === currentId);
    const rotated = at < 0 ? order : [...order.slice(at + 1), ...order.slice(0, at)];
    return rotated.find((r) => r.id !== currentId && r.status !== "filled") ?? null;
}

export function isNoteField(name: string): name is NoteField {
    return (NOTE_FIELDS as readonly string[]).includes(name);
}

/** The PUT body for a save: every field, blanks as null. */
export function toUpdateInput(operatorId: string, draft: INoteDraft) {
    return {
        operatorId,
        summary: draft.summary.trim() || null,
        pros: draft.pros || null,
        cons: draft.cons || null,
        notes: draft.notes || null,
        trivia: draft.trivia || null,
        tags: draft.tags,
    };
}

/** Replaces (or appends) `saved` in a cached notes list. */
export function upsertNote(list: readonly IOperatorNote[] | undefined, saved: IOperatorNote): IOperatorNote[] {
    const rest = (list ?? []).filter((n) => n.operator_id !== saved.operator_id);
    return [...rest, saved];
}
