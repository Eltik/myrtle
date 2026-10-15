import type { IUpsertLocaleInput } from "#/lib/api/admin";
import { GAMEDATA_SERVERS, type GamedataServer } from "#/lib/api/gamedata";
import { LOCALE_SEGMENT } from "#/lib/i18n/locale";
import type { Locale } from "#/types/generated/Locale";
import type { LocaleProgress } from "#/types/generated/LocaleProgress";

/** The Select value meaning "no fallback: straight to English". */
export const NO_FALLBACK = "__none__";

export interface ILocaleForm {
    code: string;
    englishName: string;
    nativeName: string;
    fallbackLocale: string;
    gamedataServer: GamedataServer;
    sortOrder: string;
}

export type LocaleFormField = keyof ILocaleForm;

/** Why a field is invalid. The component turns each into a sentence. */
export type LocaleFormError = "codeRequired" | "codeShape" | "codeExists" | "englishNameRequired" | "nativeNameRequired" | "fallbackSource" | "fallbackSelf" | "fallbackCycle" | "gamedataServer" | "sortOrder";

export type LocaleFormErrors = Partial<Record<LocaleFormField, LocaleFormError>>;

export function isGamedataServerCode(value: string): value is GamedataServer {
    return (GAMEDATA_SERVERS as readonly string[]).includes(value);
}

export function emptyLocaleForm(nextSortOrder: number): ILocaleForm {
    return { code: "", englishName: "", nativeName: "", fallbackLocale: NO_FALLBACK, gamedataServer: "en", sortOrder: String(nextSortOrder) };
}

export function localeForm(locale: Locale): ILocaleForm {
    return {
        code: locale.code,
        englishName: locale.english_name,
        nativeName: locale.native_name,
        fallbackLocale: locale.fallback_locale ?? NO_FALLBACK,
        gamedataServer: isGamedataServerCode(locale.gamedata_server) ? locale.gamedata_server : "en",
        sortOrder: String(locale.sort_order),
    };
}

/**
 * Walks the fallback chain from `start` and reports whether it comes back to
 * `target` (or loops anywhere). A cycle in `locales.fallback_locale` is a
 * resolver that never terminates, so the form refuses to write one.
 */
export function fallbackCycles(target: string, start: string, locales: readonly Locale[]): boolean {
    const byCode = new Map(locales.map((l) => [l.code, l]));
    const seen = new Set<string>([target]);
    let cursor: string | null = start;
    while (cursor) {
        if (seen.has(cursor)) return true;
        seen.add(cursor);
        cursor = byCode.get(cursor)?.fallback_locale ?? null;
    }
    return false;
}

/** Encodes what the rest of the system requires, not just what the columns allow. */
export function validateLocaleForm(form: ILocaleForm, locales: readonly Locale[], isNew: boolean): LocaleFormErrors {
    const errors: LocaleFormErrors = {};
    const code = form.code.trim();
    const sourceCode = locales.find((l) => l.is_source)?.code ?? null;
    const isSource = sourceCode !== null && code === sourceCode;

    if (code.length === 0) errors.code = "codeRequired";
    else if (!LOCALE_SEGMENT.test(code)) errors.code = "codeShape";
    else if (isNew && locales.some((l) => l.code === code)) errors.code = "codeExists";

    if (form.englishName.trim().length === 0) errors.englishName = "englishNameRequired";
    if (form.nativeName.trim().length === 0) errors.nativeName = "nativeNameRequired";

    if (form.fallbackLocale !== NO_FALLBACK) {
        if (isSource) errors.fallbackLocale = "fallbackSource";
        else if (form.fallbackLocale === code) errors.fallbackLocale = "fallbackSelf";
        else if (fallbackCycles(code, form.fallbackLocale, locales)) errors.fallbackLocale = "fallbackCycle";
    }

    if (!isGamedataServerCode(form.gamedataServer)) errors.gamedataServer = "gamedataServer";

    const sortOrder = Number(form.sortOrder);
    if (form.sortOrder.trim().length === 0 || !Number.isInteger(sortOrder) || sortOrder < 0) errors.sortOrder = "sortOrder";

    return errors;
}

/**
 * The upsert body. `enabled` is not on the form: a new language starts hidden,
 * an edit keeps whatever the Public switch says.
 */
export function upsertInput(form: ILocaleForm, enabled: boolean): IUpsertLocaleInput {
    return {
        code: form.code.trim(),
        englishName: form.englishName.trim(),
        nativeName: form.nativeName.trim(),
        fallbackLocale: form.fallbackLocale === NO_FALLBACK ? null : form.fallbackLocale,
        gamedataServer: form.gamedataServer,
        enabled,
        sortOrder: Number(form.sortOrder),
    };
}

/** The full row with only `enabled` changed, for the Public switch. */
export function toggleInput(locale: Locale, enabled: boolean): IUpsertLocaleInput {
    return {
        code: locale.code,
        englishName: locale.english_name,
        nativeName: locale.native_name,
        fallbackLocale: locale.fallback_locale,
        gamedataServer: locale.gamedata_server,
        enabled,
        sortOrder: locale.sort_order,
    };
}

/** Source first, then the switcher's order. */
export function sortLocales(locales: readonly Locale[]): Locale[] {
    return [...locales].sort((a, b) => Number(b.is_source) - Number(a.is_source) || a.sort_order - b.sort_order || a.code.localeCompare(b.code));
}

/** Ten past the last row, so a new language lands at the end of the switcher. */
export function nextSortOrder(locales: readonly Locale[]): number {
    return locales.length === 0 ? 0 : Math.max(...locales.map((l) => l.sort_order)) + 10;
}

/**
 * The share of the catalog translated and current (stale strings do not
 * count), as a whole percent (0-100), floored so a nearly done language never reads
 * 100%. Null when there is nothing to measure.
 */
export function translatedPercent(progress: LocaleProgress | undefined): number | null {
    if (!progress || progress.total <= 0) return null;
    const fresh = Math.max(0, progress.translated - progress.stale);
    // Integer arithmetic: `0.29 * 100` is 28.999..., which would floor to 28.
    return Math.floor((fresh * 100) / progress.total);
}
