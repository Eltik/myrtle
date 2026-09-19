import { DEFAULT_LOCALE, formatMessage, sourceMessage } from "#/lib/i18n";
import { fullMessageKey, type TypedT } from "#/lib/i18n/messages";
import { compactForSearch } from "#/lib/search/fuzzy";
import { formatNationId, rarityToNumber } from "#/lib/utils";
import type { IOperatorListItem } from "#/types/operators";
import { NON_OPERATOR_PROFESSIONS } from "./constants";
import type { messages as helperMessages } from "./helpers.messages";
import type { CalendarScale, IBirthdayFilters, IOperatorBirthday, ISelectedDay } from "./types";

/** The `t` the range-title helper needs, narrowed to the keys it can render. */
export type BirthdayHelperT = TypedT<typeof helperMessages>;

/**
 * Default `t` for a caller outside an `I18nProvider`. It resolves against the
 * bundled source catalog, so the English is the same one the components render
 * and this file carries no second copy of the text.
 */
const sourceT: BirthdayHelperT = (key, values) => formatMessage(sourceMessage(fullMessageKey("tools", key)) ?? key, DEFAULT_LOCALE, values);

/**
 * Month names for a locale, index 0 = January. `short` gives the abbreviated
 * form. These come from `Intl` rather than the message catalog: a locale the
 * catalog has never seen still gets its own month names.
 */
export function monthNames(locale: string, short = false): string[] {
    const fmt = new Intl.DateTimeFormat(locale, { month: short ? "short" : "long", timeZone: "UTC" });
    return Array.from({ length: 12 }, (_, i) => fmt.format(new Date(Date.UTC(2021, i, 15))));
}

/** Abbreviated weekday names for a locale, index 0 = Sunday (the grid is Sunday-first). */
export function weekdayNames(locale: string): string[] {
    const fmt = new Intl.DateTimeFormat(locale, { weekday: "short", timeZone: "UTC" });
    // 2021-08-01 was a Sunday, so the seven days from it start on Sunday.
    return Array.from({ length: 7 }, (_, i) => fmt.format(new Date(Date.UTC(2021, 7, 1 + i))));
}

/** Numeric rarity (1-6) for an operator. */
export function operatorRarity(operator: IOperatorListItem): number {
    return rarityToNumber(operator.rarity);
}

/** The themed CSS color token for a rarity tier, e.g. `var(--rarity-6)`. */
export function rarityVar(rarity: number): string {
    return `var(--rarity-${rarity})`;
}

/** Count of operators with a parseable birthday. */
export function countKnown(birthdays: IOperatorBirthday[]): number {
    return birthdays.reduce((n, b) => n + (b.known ? 1 : 0), 0);
}

/** Drop summons and map hazards - they aren't operators with birthdays. */
export function isCalendarOperator(operator: IOperatorListItem): boolean {
    return !NON_OPERATOR_PROFESSIONS.has(operator.profession);
}

/** Whole days from `today` until the next occurrence of month/day (0 = today). `month` is 1-12. */
export function daysUntil(month: number, day: number, today: Date): number {
    const anchor = new Date(today.getFullYear(), today.getMonth(), today.getDate());
    let target = new Date(anchor.getFullYear(), month - 1, day);
    if (target < anchor) target = new Date(anchor.getFullYear() + 1, month - 1, day);
    return Math.round((target.getTime() - anchor.getTime()) / 86_400_000);
}

/** Orders known birthdays by how soon they recur from `today`, today first. */
export function compareUpcoming(a: { month: number; day: number }, b: { month: number; day: number }, today: Date): number {
    return daysUntil(a.month, a.day, today) - daysUntil(b.month, b.day, today);
}

export interface ICalendarCell {
    year: number;
    /** 1-12 */
    month: number;
    day: number;
    /** Day belongs to the previous/next month, shown faded for grid alignment. */
    outside: boolean;
}

/** A fixed 6-week (42 cell) month grid, Sunday-first, with leading/trailing days from adjacent months. */
export function buildCalendarCells(year: number, month: number): ICalendarCell[] {
    const startWeekday = new Date(year, month - 1, 1).getDay();
    const daysInMonth = new Date(year, month, 0).getDate();
    const daysInPrev = new Date(year, month - 1, 0).getDate();

    const cells: ICalendarCell[] = [];
    for (let i = startWeekday - 1; i >= 0; i--) {
        cells.push({ year: month === 1 ? year - 1 : year, month: month === 1 ? 12 : month - 1, day: daysInPrev - i, outside: true });
    }
    for (let d = 1; d <= daysInMonth; d++) cells.push({ year, month, day: d, outside: false });
    for (let d = 1; cells.length < 42; d++) {
        cells.push({ year: month === 12 ? year + 1 : year, month: month === 12 ? 1 : month + 1, day: d, outside: true });
    }
    return cells;
}

/** A fresh, fully-cleared filter set (own Set instances, safe as initial state). */
export function createEmptyFilters(): IBirthdayFilters {
    return { query: "", rarities: new Set(), professions: new Set(), nations: new Set() };
}

/** How many filter facets are currently active. */
export function countActiveFilters(filters: IBirthdayFilters): number {
    return filters.rarities.size + filters.professions.size + filters.nations.size + (filters.query.trim() ? 1 : 0);
}

export function applyFilters(birthdays: IOperatorBirthday[], filters: IBirthdayFilters): IOperatorBirthday[] {
    const query = compactForSearch(filters.query);
    return birthdays.filter((b) => {
        const op = b.operator;
        if (filters.rarities.size > 0 && !filters.rarities.has(operatorRarity(op))) return false;
        if (filters.professions.size > 0 && !filters.professions.has(op.profession)) return false;
        if (filters.nations.size > 0 && !filters.nations.has(op.nationId)) return false;
        if (query && !compactForSearch(op.name).includes(query)) return false;
        return true;
    });
}

/** "month-day" key for fast calendar-cell and day-panel lookup. */
export function dayKey(month: number, day: number): string {
    return `${month}-${day}`;
}

/** Birthdays falling on a given month/day in a grouped map, or an empty list. */
export function opsOn(byDay: Map<string, IOperatorBirthday[]>, month: number, day: number): IOperatorBirthday[] {
    return byDay.get(dayKey(month, day)) ?? [];
}

/** Groups known birthdays by `dayKey`, each bucket sorted by rarity desc then name. */
export function groupByDay(birthdays: IOperatorBirthday[]): Map<string, IOperatorBirthday[]> {
    const map = new Map<string, IOperatorBirthday[]>();
    for (const b of birthdays) {
        if (!b.known) continue;
        const key = dayKey(b.month, b.day);
        const bucket = map.get(key);
        if (bucket) bucket.push(b);
        else map.set(key, [b]);
    }
    for (const bucket of map.values()) {
        bucket.sort((a, b) => operatorRarity(b.operator) - operatorRarity(a.operator) || a.operator.name.localeCompare(b.operator.name));
    }
    return map;
}

/** Distinct nations present in the data, as `[nationId, label]` sorted by label. */
export function deriveNations(operators: IOperatorListItem[]): [string, string][] {
    const ids = new Set<string>();
    for (const op of operators) {
        if (op.nationId) ids.add(op.nationId);
    }
    return [...ids].map((id) => [id, formatNationId(id)] as [string, string]).sort((a, b) => a[1].localeCompare(b[1]));
}

export function addDays(date: Date, days: number): Date {
    const next = new Date(date);
    next.setDate(next.getDate() + days);
    return next;
}

export function addMonths(date: Date, months: number): Date {
    const next = new Date(date);
    next.setDate(1); // avoid month-end overflow (e.g. Jan 31 -> Mar)
    next.setMonth(next.getMonth() + months);
    return next;
}

/** Sunday-anchored start of the week containing `date`, time zeroed. */
export function startOfWeek(date: Date): Date {
    const next = new Date(date.getFullYear(), date.getMonth(), date.getDate() - date.getDay());
    return next;
}

export function isSameDate(a: Date, b: Date): boolean {
    return a.getFullYear() === b.getFullYear() && a.getMonth() === b.getMonth() && a.getDate() === b.getDate();
}

/** Whether a specific calendar date (1-12 month) is today. */
export function isToday(today: Date, year: number, month: number, day: number): boolean {
    return isSameDate(today, new Date(year, month - 1, day));
}

/** Whether a recurring birthday (month/day, any year) lands on today. */
export function isTodayMonthDay(today: Date, month: number, day: number): boolean {
    return today.getMonth() + 1 === month && today.getDate() === day;
}

/** Convert a `Date` to a `{ year, month, day }` (1-12 month). */
export function toSelectedDay(date: Date): ISelectedDay {
    return { year: date.getFullYear(), month: date.getMonth() + 1, day: date.getDate() };
}

/** Number of day-columns shown for an agenda scale (month is grid-based, not listed here). */
const AGENDA_LENGTH: Record<Exclude<CalendarScale, "month">, number> = { day: 1, "3day": 3, week: 7 };

/** The ordered days an agenda scale renders. Week snaps to its Sunday; day/3day start at the anchor. */
export function buildAgendaDays(scale: Exclude<CalendarScale, "month">, anchor: Date): Date[] {
    const start = scale === "week" ? startOfWeek(anchor) : new Date(anchor.getFullYear(), anchor.getMonth(), anchor.getDate());
    return Array.from({ length: AGENDA_LENGTH[scale] }, (_, i) => addDays(start, i));
}

/** Moves the anchor one page in `dir` (±1) at the current scale's granularity. */
export function stepAnchor(scale: CalendarScale, anchor: Date, dir: number): Date {
    switch (scale) {
        case "day":
            return addDays(anchor, dir);
        case "3day":
            return addDays(anchor, dir * 3);
        case "week":
            return addDays(anchor, dir * 7);
        case "month":
            return addMonths(anchor, dir);
    }
}

/**
 * The header label for the visible range, e.g. "October 2026" or "Oct 26 - Nov 1, 2026".
 * `short` drops the weekday and abbreviates months so the title fits on small screens.
 */
export function formatRangeTitle(scale: CalendarScale, anchor: Date, locale: string = DEFAULT_LOCALE, t: BirthdayHelperT = sourceT, short = false): string {
    const long = monthNames(locale);
    const abbr = monthNames(locale, true);
    if (scale === "month") return t("birthdays.range.month", { month: (short ? abbr : long)[anchor.getMonth()], year: anchor.getFullYear() });
    const days = buildAgendaDays(scale, anchor);
    const start = days[0];
    if (days.length === 1) {
        return short ? t("birthdays.range.day", { month: abbr[start.getMonth()], day: start.getDate(), year: start.getFullYear() }) : new Intl.DateTimeFormat(locale, { weekday: "long", month: "long", day: "numeric", year: "numeric" }).format(start);
    }

    const end = days[days.length - 1];
    if (start.getFullYear() !== end.getFullYear()) {
        return t("birthdays.range.crossYear", {
            startMonth: abbr[start.getMonth()],
            startDay: start.getDate(),
            startYear: start.getFullYear(),
            endMonth: abbr[end.getMonth()],
            endDay: end.getDate(),
            endYear: end.getFullYear(),
        });
    }
    if (start.getMonth() !== end.getMonth()) {
        return t("birthdays.range.crossMonth", {
            startMonth: abbr[start.getMonth()],
            startDay: start.getDate(),
            endMonth: abbr[end.getMonth()],
            endDay: end.getDate(),
            year: end.getFullYear(),
        });
    }
    return t("birthdays.range.sameMonth", { month: long[start.getMonth()], startDay: start.getDate(), endDay: end.getDate(), year: end.getFullYear() });
}
