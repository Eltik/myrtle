/**
 * The Translations section's pure logic: status, progress arithmetic, the
 * placeholder rule, list navigation and the ICU form table. No React and no
 * session, so every rule here is unit-tested on its own.
 */
import { currentPercent, missingCount } from "#/components/admin/shell/model";
import type { AdminTranslationsFilter } from "#/components/admin/shell/search";
import { describeMessage, type IMessageArgument, pluralCategoriesFor, pluralExamples } from "#/lib/i18n/format";
import type { JsonValue } from "#/types/generated/serde_json/JsonValue";

export type EntryStatus = "translated" | "stale" | "untranslated";

export function entryStatus(entry: { value: string | null; is_stale: boolean }): EntryStatus {
    if (entry.value === null) return "untranslated";
    return entry.is_stale ? "stale" : "translated";
}

export interface ILocaleStats {
    total: number;
    stale: number;
    missing: number;
    /** Out of date plus missing: what the "To do" filter lists. */
    todo: number;
    /** Share of strings translated and current, floored to one decimal (design `pct`). */
    pct: number;
}

/**
 * Progress for one locale card. A string that is translated but out of date
 * does not count as done, so the bar only fills once the English and the
 * translation agree again. An empty catalog counts as complete.
 */
export function localeStats(progress: { total: number; translated: number; stale: number } | undefined): ILocaleStats {
    if (!progress) return { total: 0, stale: 0, missing: 0, todo: 0, pct: 0 };
    const missing = missingCount(progress);
    const stale = Math.max(0, progress.stale);
    const pct = currentPercent({ total: progress.total, translated: progress.translated, stale });
    return { total: progress.total, stale, missing, todo: stale + missing, pct };
}

/** The filter a missing `filter` param means: work through the backlog if you can edit, else browse everything. */
export function resolveFilter(requested: AdminTranslationsFilter | undefined, canEdit: boolean): AdminTranslationsFilter {
    return requested ?? (canEdit ? "todo" : "all");
}

/** The requested locale if this user can see it, else their first one. */
export function resolveLocale(visible: readonly string[], requested: string | undefined): string | undefined {
    if (requested && visible.includes(requested)) return requested;
    return visible[0];
}

/**
 * `ui_message_keys.placeholders` is a JSON array of argument names. Objects
 * are tolerated so an older row shape never blanks the chip row.
 */
export function placeholderNames(raw: JsonValue): string[] {
    if (Array.isArray(raw)) return raw.filter((v): v is string => typeof v === "string");
    if (raw !== null && typeof raw === "object") return Object.keys(raw);
    return [];
}

/**
 * Every argument the English uses, parsed ones first, then any the backend
 * declares that the parser did not reach. The backend rejects a save that
 * drops a declared one, so it has to be on screen to be explainable.
 */
export function mergeArguments(parsed: readonly IMessageArgument[], declared: readonly string[]): IMessageArgument[] {
    const seen = new Set(parsed.map((a) => a.name));
    const extra = declared.filter((name) => !seen.has(name)).map((name): IMessageArgument => ({ branchText: {}, branches: [], name, offset: null, type: "plain" }));
    return [...parsed, ...extra];
}

/** The arguments of a source string, parsed and declared. */
export function sourceArguments(source: string, declared: JsonValue): IMessageArgument[] {
    return mergeArguments(describeMessage(source), placeholderNames(declared));
}

function escapeRegExp(text: string): string {
    return text.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

/**
 * Whether the draft still carries `{name}`, as a plain argument (`{name}`) or
 * as the head of a formatted one (`{name, plural, ...}`).
 */
export function hasPlaceholder(draft: string, name: string): boolean {
    return new RegExp(`\\{\\s*${escapeRegExp(name)}\\s*[},]`).test(draft);
}

export function missingPlaceholders(names: readonly string[], draft: string): string[] {
    return names.filter((name) => !hasPlaceholder(draft, name));
}

/**
 * The selected row: the explicit selection when it is in the list, else the
 * first row (design `curKey`).
 */
export function selectEntry<T extends { key: string }>(list: readonly T[], key: string | undefined): T | undefined {
    return (key ? list.find((e) => e.key === key) : undefined) ?? list[0];
}

/**
 * Where Save & next lands: the row after, or the one before when this was the
 * last. `undefined` when nothing else is listed.
 */
export function keyAfterSave(list: readonly { key: string }[], key: string): string | undefined {
    const i = list.findIndex((e) => e.key === key);
    if (i < 0) return list[0]?.key;
    return list[i + 1]?.key ?? (i > 0 ? list[i - 1].key : undefined);
}

/** Skip moves to the next row and wraps to the top. */
export function keyAfterSkip(list: readonly { key: string }[], key: string): string | undefined {
    if (list.length === 0) return undefined;
    const i = list.findIndex((e) => e.key === key);
    return list[(i + 1) % list.length].key;
}

/**
 * Flattens loaded pages, keeping the first copy of a key. A save can shift the
 * backend's offsets between two page fetches, which would otherwise repeat a
 * row at the page seam.
 */
export function flattenPages<T extends { key: string }>(pages: readonly { entries: readonly T[] }[]): T[] {
    const seen = new Set<string>();
    const out: T[] = [];
    for (const page of pages) {
        for (const entry of page.entries) {
            if (seen.has(entry.key)) continue;
            seen.add(entry.key);
            out.push(entry);
        }
    }
    return out;
}

export function isBranching(arg: IMessageArgument): boolean {
    return arg.type === "plural" || arg.type === "selectordinal" || arg.type === "select";
}

export type FormStatus = "written" | "missing" | "unused";

export interface IFormRow {
    key: string;
    /** The numbers that select this form, the exact value of an `=N` form, or `""` for `other`. */
    applies: string;
    english: string;
    status: FormStatus;
}

/**
 * One row per wording the translator has to think about.
 *
 * The required set comes from the TARGET locale, not English: English declares
 * `one` and `other`, Russian needs `one`, `few`, `many` and `other`, and a
 * Russian translation that stops at two forms is silently wrong.
 */
export function formRows(arg: IMessageArgument, written: IMessageArgument | null, locale: string, started: boolean): IFormRow[] {
    const numeric = arg.type === "plural" || arg.type === "selectordinal";
    const ruleType = arg.type === "selectordinal" ? "selectordinal" : "plural";
    const writtenKeys = new Set(written?.branches ?? []);

    const exact = [...new Set([...arg.branches, ...(written?.branches ?? [])])].filter((k) => k.startsWith("=")).sort((a, b) => Number(a.slice(1)) - Number(b.slice(1)));
    const named = numeric ? pluralCategoriesFor(locale, ruleType) : arg.branches.filter((k) => !k.startsWith("="));
    const spare = [...writtenKeys, ...arg.branches].filter((k) => !k.startsWith("=") && !named.includes(k));
    const order = [...exact, ...named, ...new Set(spare)];

    const required = new Set<string>(numeric ? [...named, ...arg.branches.filter((k) => k.startsWith("="))] : arg.branches);
    const examples = new Map(numeric ? pluralExamples(locale, ruleType).map((e) => [e.category, e.examples]) : []);

    return order.map((key) => ({
        applies: key.startsWith("=") ? key.slice(1) : key === "other" ? "" : (examples.get(key) ?? []).join(", "),
        english: arg.branchText[key] ?? "",
        key,
        status: writtenKeys.has(key) ? "written" : started && required.has(key) ? "missing" : "unused",
    }));
}
