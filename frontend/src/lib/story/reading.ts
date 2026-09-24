/**
 * READING SPEED IS A SETTING, NOT A MEASUREMENT. Nothing in the index says how
 * fast a reader reads, so every span this module prints is a word count divided
 * by a number the reader chose. The default, 225 words a minute, is the middle
 * of the 200 to 250 band the reading-time literature quotes for adult prose;
 * it is a trade, shipped knowingly, and the input beside every figure is what
 * makes it one rather than a claim.
 *
 * The bounds are the same trade. 40 is slower than a fluent adult reads aloud
 * and 2,000 is faster than a skimmer sustains, so they fence off the values
 * that would print an absurd span rather than asserting where real reading
 * stops.
 *
 * Everything here is pure but {@link useReadingSpeed}, so the arithmetic is
 * testable without a DOM and the two persisted numbers have exactly one owner.
 */

import { useLocalStorageState } from "#/hooks/use-local-storage-state";
import { clamp } from "./num";

/** Both keys carry the site prefix every other document in this browser does (`myrtle.story.settings`, `myrtle.story.progress`), so a reader clearing this site's state can recognise them. */
export const WPM_KEY = "myrtle.story.reading.wpm";
export const MINUTES_PER_DAY_KEY = "myrtle.story.reading.minutesPerDay";

export const DEFAULT_WPM = 225;
export const MIN_WPM = 40;
export const MAX_WPM = 2000;

export const DEFAULT_MINUTES_PER_DAY = 120;
export const MIN_MINUTES_PER_DAY = 5;
export const MAX_MINUTES_PER_DAY = 960;

/** Minutes in a day, the boundary between the `h` and the `d` shape. */
const MINUTES_PER_DAY = 1440;

/** Days in an average Gregorian year over twelve, 365.2425 / 12. A month is the unit the "left to go" line speaks in, and 30 would drift a full day every quarter. */
export const DAYS_PER_MONTH = 30.436875;

const MINUTES_PER_MONTH = DAYS_PER_MONTH * MINUTES_PER_DAY;

/**
 * Coerce a stored or typed value into a reading speed.
 *
 * A MISSING value is checked for explicitly rather than falsily, because
 * `Number(null)` is 0 and finite and would clamp to 40 instead of falling back
 * to the default.
 */
export function clampWpm(value: unknown): number {
    const n = typeof value === "number" ? value : typeof value === "string" && value.trim() !== "" ? Number(value) : Number.NaN;
    if (!Number.isFinite(n)) return DEFAULT_WPM;
    return clamp(Math.round(n), MIN_WPM, MAX_WPM);
}

/** The same coercion for the daily reading budget. */
export function clampMinutesPerDay(value: unknown): number {
    const n = typeof value === "number" ? value : typeof value === "string" && value.trim() !== "" ? Number(value) : Number.NaN;
    if (!Number.isFinite(n)) return DEFAULT_MINUTES_PER_DAY;
    return clamp(Math.round(n), MIN_MINUTES_PER_DAY, MAX_MINUTES_PER_DAY);
}

/** Minutes a span of prose takes at `wpm`. A non-positive or absent count is 0 minutes, never NaN. */
export function minutesFor(words: number, wpm: number): number {
    if (!Number.isFinite(words) || words <= 0) return 0;
    const rate = clampWpm(wpm);
    return words / rate;
}

/**
 * A span of minutes as the shortest thing a reader can picture: `31m`,
 * `7h 28m`, `1d 6h`, `6.0 months`.
 *
 * The shape steps up at the unit above, not at a round number: minutes up to an
 * hour, hours up to a day, days up to a month, months beyond. Two units are
 * printed at most, because the third is noise at every scale this library
 * reaches, and the smaller one is dropped when it is zero rather than printed
 * as `7h 0m`.
 *
 * The unit letters are NOT translated. They are symbols on a tabular-nums
 * figure, the way the `1,234` beside them is, and splitting them into four
 * message keys would buy a translator four strings of one letter each.
 */
export function humanTime(minutes: number): string {
    if (!Number.isFinite(minutes) || minutes <= 0) return "0m";
    const whole = Math.round(minutes);
    if (whole < 1) return "<1m";
    if (whole < 60) return `${whole}m`;
    if (whole < MINUTES_PER_DAY) {
        const hours = Math.floor(whole / 60);
        const rest = whole % 60;
        return rest === 0 ? `${hours}h` : `${hours}h ${rest}m`;
    }
    if (whole < MINUTES_PER_MONTH) {
        const hours = Math.round(whole / 60);
        const days = Math.floor(hours / 24);
        const rest = hours % 24;
        return rest === 0 ? `${days}d` : `${days}d ${rest}h`;
    }
    return `${(whole / MINUTES_PER_MONTH).toFixed(1)} months`;
}

/**
 * `m:ss`, which is how long a piece of MUSIC is written everywhere it is
 * written. It sits beside {@link humanTime} rather than inside the player
 * because the two are the one family: a span the reader is asked to plan with
 * reads `7h 28m`, and a span they are asked to scrub reads `3:04`.
 *
 * There is no hour field. The longest cue in the bank is under ten minutes, so
 * a cue that somehow ran past one reads 71:30 rather than lie about it.
 */
export function clockTime(seconds: number): string {
    const whole = Number.isFinite(seconds) ? Math.max(0, Math.floor(seconds)) : 0;
    const s = whole % 60;
    return `${Math.floor(whole / 60)}:${s < 10 ? "0" : ""}${s}`;
}

/** Whether a pace line speaks in days or in months, and the figure it prints. */
export interface IPaceSpan {
    unit: "day" | "month";
    value: number;
}

/**
 * How long a span of reading lasts at a daily budget.
 *
 * Under two months it reads in DAYS, because "0.4 months" is a figure nobody
 * plans with, and beyond that in months to one decimal.
 */
export function paceSpan(minutes: number, minutesPerDay: number): IPaceSpan {
    const budget = clampMinutesPerDay(minutesPerDay);
    const days = Number.isFinite(minutes) && minutes > 0 ? minutes / budget : 0;
    if (days < 2 * DAYS_PER_MONTH) return { unit: "day", value: Math.max(0, Math.round(days)) };
    return { unit: "month", value: Math.round((days / DAYS_PER_MONTH) * 10) / 10 };
}

export interface IReadingSpeed {
    wpm: number;
    setWpm: (next: number) => void;
    minutesPerDay: number;
    setMinutesPerDay: (next: number) => void;
}

/**
 * The reader's two numbers, persisted per browser.
 *
 * They are NOT part of `myrtle.story.progress`: progress is the document that
 * syncs to the account and merges across devices, and a reading speed is a
 * preference of this browser's owner, not a fact about what they have read.
 * Sending it through the merge would give it a last-write-wins race for no gain.
 */
export function useReadingSpeed(): IReadingSpeed {
    const [wpm, setWpmRaw] = useLocalStorageState<number>(WPM_KEY, DEFAULT_WPM, { parse: (raw) => clampWpm(JSON.parse(raw)) });
    const [minutesPerDay, setMinutesPerDayRaw] = useLocalStorageState<number>(MINUTES_PER_DAY_KEY, DEFAULT_MINUTES_PER_DAY, { parse: (raw) => clampMinutesPerDay(JSON.parse(raw)) });
    return {
        wpm: clampWpm(wpm),
        setWpm: (next) => setWpmRaw(clampWpm(next)),
        minutesPerDay: clampMinutesPerDay(minutesPerDay),
        setMinutesPerDay: (next) => setMinutesPerDayRaw(clampMinutesPerDay(next)),
    };
}
