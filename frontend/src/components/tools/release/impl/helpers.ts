import { env } from "#/env";
import { DEFAULT_LOCALE, formatMessage, sourceMessage } from "#/lib/i18n";
import { fullMessageKey, type TypedT } from "#/lib/i18n/messages";
import type { LagModel } from "#/types/generated/LagModel";
import type { Resolution } from "#/types/generated/Resolution";
import type { messages as helperMessages } from "./helpers.messages";

/** The `t` these helpers need, narrowed to the keys they can render. */
export type ReleaseHelperT = TypedT<typeof helperMessages>;

/**
 * Default `t` for a caller outside an `I18nProvider`. It resolves against the
 * bundled source catalog, so the English is the same one the components render
 * and this file carries no second copy of the text.
 */
const sourceT: ReleaseHelperT = (key, values) => formatMessage(sourceMessage(fullMessageKey("tools", key)) ?? key, DEFAULT_LOCALE, values);

const DAY_MS = 86_400_000;

export const PAST_GRACE_DAYS = 14;

/**
 * Abbreviated weekday names for a locale, index 0 = Sunday (the calendar grid
 * is Sunday-first). These come from `Intl` rather than the message catalog: a
 * locale the catalog has never seen still gets its own weekday names.
 */
export function weekdayNames(locale: string): string[] {
    const fmt = new Intl.DateTimeFormat(locale, { weekday: "short", timeZone: "UTC" });
    // 2021-08-01 was a Sunday, so the seven days from it start on Sunday.
    return Array.from({ length: 7 }, (_, i) => fmt.format(new Date(Date.UTC(2021, 7, 1 + i))));
}

export function fromUnix(seconds: number): Date {
    return new Date(seconds * 1000);
}

/** The locale is an argument rather than a hook so non-React callers can use it. */
export function formatDate(seconds: number, locale?: string): string {
    return fromUnix(seconds).toLocaleDateString(locale ?? "en-US", { month: "short", day: "numeric", year: "numeric" });
}

function formatMonthDay(seconds: number, locale?: string): string {
    return fromUnix(seconds).toLocaleDateString(locale ?? "en-US", { month: "short", day: "numeric" });
}

export function formatDateRange(start: number, end: number | null | undefined, locale?: string, t: ReleaseHelperT = sourceT): string {
    if (!end || end <= start) return formatDate(start, locale);
    const a = fromUnix(start);
    const b = fromUnix(end);
    if (a.getFullYear() === b.getFullYear()) return t("release.dateRange", { start: formatMonthDay(start, locale), end: formatDate(end, locale) });
    return t("release.dateRange", { start: formatDate(start, locale), end: formatDate(end, locale) });
}

export function daysFromToday(seconds: number, today: Date): number {
    const anchor = new Date(today.getFullYear(), today.getMonth(), today.getDate());
    const d = fromUnix(seconds);
    const target = new Date(d.getFullYear(), d.getMonth(), d.getDate());
    return Math.round((target.getTime() - anchor.getTime()) / DAY_MS);
}

export function relativeDays(days: number, t: ReleaseHelperT = sourceT): string {
    if (days === 0) return t("release.rel.today");
    if (days === 1) return t("release.rel.tomorrow");
    if (days === -1) return t("release.rel.yesterday");
    return days > 0 ? t("release.rel.inDays", { count: days }) : t("release.rel.daysAgo", { count: -days });
}

export function resolvedEnStart(r: Resolution): number | null {
    return r.status === "unmodelled" || r.status === "independent" ? null : r.enStart;
}

export function sortKey(r: Resolution, cnStart: number, model: LagModel | null | undefined): number {
    const en = resolvedEnStart(r);
    if (en !== null) return en;
    return cnStart + (model?.medianDays ?? 0) * 86_400;
}

export function isPast(keySeconds: number, today: Date): boolean {
    return daysFromToday(keySeconds, today) < -PAST_GRACE_DAYS;
}

export function formatDays(n: number): string {
    return Number.isInteger(n) ? String(n) : n.toFixed(1);
}

export function formatPercent(fraction: number): string {
    return `${Math.round(fraction * 100)}%`;
}

export function humanizeTag(raw: string): string {
    const s = raw.replace(/_/g, " ").toLowerCase();
    return s.charAt(0).toUpperCase() + s.slice(1);
}

export function assetUrl(path: string | null): string | null {
    if (!path) return null;
    const encoded = path.split("/").map(encodeURIComponent).join("/");
    return `${env.VITE_BACKEND_URL ?? ""}/api${encoded.startsWith("/") ? "" : "/"}${encoded}`;
}
