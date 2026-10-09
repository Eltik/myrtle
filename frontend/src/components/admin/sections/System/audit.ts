import type { TextField } from "#/components/admin/sections/OperatorNotes/notesModel";
import type { IAuditLogEntry } from "#/lib/api/admin";
import { normalizeForSearch } from "#/lib/search/fuzzy";
import type { TranslationAuditEntry } from "#/types/generated/TranslationAuditEntry";

/** Which log a timeline row came from. */
export type AuditSource = "notes" | "translations";

/** The design's All / Operator notes / Translations switch. */
export type AuditSourceFilter = "all" | AuditSource;

/** `"all"`, one note field, or `"text"` (a translation's only field). */
export type AuditFieldFilter = "all" | TextField | "text";

export interface IAuditActor {
    userId: string;
    /** Null once the account has been hard-deleted. */
    uid: string | null;
    nickname: string | null;
}

/** One row of the merged edit history, whichever log it came from. */
export interface IAuditRow {
    /** Unique across both logs: the two id sequences overlap. */
    key: string;
    source: AuditSource;
    changedAt: string;
    actor: IAuditActor;
    /** Note rows: the operator. */
    operatorId: string | null;
    /** Translation rows: the locale code and message key. */
    locale: string | null;
    messageKey: string | null;
    /** Note rows: the audited column. Translation rows: `"text"`. */
    field: string;
    oldValue: string | null;
    newValue: string | null;
}

function blank(value: string | null): boolean {
    return value == null || value.trim().length === 0;
}

/** The new value (or, for a clear, the old one) on one line, cut to `max` characters. */
export function previewValue(oldValue: string | null, newValue: string | null, max = 80): string {
    const source = blank(newValue) ? (oldValue ?? "") : (newValue ?? "");
    const flat = source.replace(/\s*\n\s*/g, " ").trim();
    return flat.length > max ? `${flat.slice(0, max).trimEnd()}…` : flat;
}

export function noteRow(entry: IAuditLogEntry): IAuditRow {
    return {
        key: `n${entry.id}`,
        source: "notes",
        changedAt: entry.changed_at,
        actor: { userId: entry.actor.user_id, uid: entry.actor.uid, nickname: entry.actor.nickname },
        operatorId: entry.operator_id,
        locale: null,
        messageKey: null,
        field: entry.field_name,
        oldValue: entry.old_value,
        newValue: entry.new_value,
    };
}

export function translationRow(entry: TranslationAuditEntry): IAuditRow {
    return {
        key: `t${entry.id}`,
        source: "translations",
        changedAt: entry.changed_at,
        actor: { userId: entry.actor.user_id, uid: entry.actor.uid, nickname: entry.actor.nickname },
        operatorId: null,
        locale: entry.locale,
        messageKey: entry.message_key,
        field: "text",
        oldValue: entry.old_value,
        newValue: entry.new_value,
    };
}

export interface ITimelineInput {
    notes: readonly IAuditLogEntry[];
    translations: readonly TranslationAuditEntry[];
    /** The notes log has older pages not loaded yet. */
    notesHasMore: boolean;
    translationsHasMore: boolean;
    source: AuditSourceFilter;
}

export interface ITimeline {
    /** Newest first, cut where the timeline stops being complete. */
    rows: IAuditRow[];
    /** Loaded rows held back because an older page of the other log could interleave with them. */
    held: number;
}

function time(iso: string): number {
    const t = Date.parse(iso);
    return Number.isNaN(t) ? 0 : t;
}

/**
 * Merges the two logs newest first. Each log is paged on its own, so below the
 * oldest loaded row of a log that has more pages, the other log's rows could
 * be missing their neighbours. Those rows are held back (not dropped: the next
 * page shows them) so the timeline never shows a gap as if it were quiet.
 */
export function buildTimeline({ notes, translations, notesHasMore, translationsHasMore, source }: ITimelineInput): ITimeline {
    const useNotes = source !== "translations";
    const useTranslations = source !== "notes";
    const rows = [...(useNotes ? notes.map(noteRow) : []), ...(useTranslations ? translations.map(translationRow) : [])];
    rows.sort((a, b) => time(b.changedAt) - time(a.changedAt) || a.source.localeCompare(b.source) || b.key.localeCompare(a.key, undefined, { numeric: true }));

    let cutoff = Number.NEGATIVE_INFINITY;
    const oldest = (list: readonly { changed_at: string }[]): number => (list.length ? time(list[list.length - 1].changed_at) : Number.NEGATIVE_INFINITY);
    if (useNotes && notesHasMore) cutoff = Math.max(cutoff, oldest(notes));
    if (useTranslations && translationsHasMore) cutoff = Math.max(cutoff, oldest(translations));

    const shown = rows.filter((r) => time(r.changedAt) >= cutoff);
    return { rows: shown, held: rows.length - shown.length };
}

/**
 * Narrows the timeline by field and free text. `labels` supplies the display
 * strings a reader searches by (operator name, language name), keyed by row.
 */
export function filterTimeline(rows: readonly IAuditRow[], field: AuditFieldFilter, query: string, labels: (row: IAuditRow) => readonly string[]): IAuditRow[] {
    const q = normalizeForSearch(query.trim());
    return rows.filter((r) => {
        if (field !== "all" && r.field !== field) return false;
        if (!q) return true;
        const haystack = [r.actor.nickname, r.actor.uid, r.operatorId, r.locale, r.messageKey, r.field, r.oldValue, r.newValue, ...labels(r)];
        return haystack.some((s) => s != null && normalizeForSearch(s).includes(q));
    });
}
