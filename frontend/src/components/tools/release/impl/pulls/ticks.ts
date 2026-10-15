/**
 * X-axis tick picking for the Pulls tab's category charts, done here instead of by Recharts.
 *
 * Recharts' default `interval` ("preserveEnd") sizes EVERY candidate label in the DOM to
 * honour `minTickGap`: it writes the text into a hidden span and reads it back with
 * `getBoundingClientRect`, once per label, and the axis and the grid each run that pass.
 * On this site each such write is a full-document style recalculation (about 17 ms, from
 * the `:has()` rules in the global CSS), so the odds curve's 151 labels and the
 * simulator's buckets cost 385 forced recalculations, 5.7 s, the first time the tab
 * opened. A numeric `interval` skips the measuring entirely, so the charts get their
 * labels from `endAnchoredTicks` and pass `interval={0}`.
 */

/** Pixels one digit of the 11 px axis font takes, rounded up from Recharts' own measurement (6.1 px). */
const DIGIT_PX = 6.5;

/** The `minTickGap` the category charts used before, kept as the spacing between two labels. */
const MIN_TICK_GAP = 30;

/** The `minTickGap` the budget chart's date axis used before. */
const DATE_TICK_GAP = 40;

const DAY_MS = 86_400_000;

/** Width the axis loses to the Y axis and the margins, so labels spread over the plot only. */
const PLOT_INSET = 40;

/** How many rows apart two labels must sit for `labelChars`-wide labels to clear the gap in `width` px. */
export function labelStride(count: number, width: number, labelChars: number, gap = MIN_TICK_GAP): number {
    if (count <= 1) return 1;
    const plot = Math.max(1, width - PLOT_INSET);
    const pxPerRow = plot / (count - 1);
    const needed = labelChars * DIGIT_PX + gap;
    return Math.max(1, Math.ceil(needed / pxPerRow));
}

/**
 * Every `stride`-th value counted back from the last, in ascending order: the last row
 * always carries a label, as it did under Recharts' "preserveEnd".
 */
export function endAnchoredTicks<T>(values: readonly T[], stride: number): T[] {
    const step = Math.max(1, Math.floor(stride));
    const out: T[] = [];
    for (let i = values.length - 1; i >= 0; i -= step) out.push(values[i] as T);
    return out.reverse();
}

/** The labels for a category axis over `values`, laid out for a chart `width` px wide. */
export function categoryTicks<T extends string | number>(values: readonly T[], width: number): T[] {
    const labelChars = values.reduce((max, v) => Math.max(max, String(v).length), 1);
    return endAnchoredTicks(values, labelStride(values.length, width, labelChars));
}

/**
 * Day-aligned labels for a time axis running `min`..`max` (epoch ms), counted back from
 * `max` in whole days so the last day always carries a label, spaced so `labelChars`-wide
 * dates clear the budget chart's 40 px gap.
 */
export function timeTicks(min: number, max: number, width: number, labelChars: number): number[] {
    if (!(max > min)) return Number.isFinite(max) ? [max] : [];
    const spanDays = (max - min) / DAY_MS;
    const stepDays = labelStride(Math.floor(spanDays) + 1, width, labelChars, DATE_TICK_GAP);
    const out: number[] = [];
    for (let at = max; at >= min; at -= stepDays * DAY_MS) out.push(at);
    return out.reverse();
}

/**
 * For `CartesianGrid`'s `verticalCoordinatesGenerator` on a grid drawn with `vertical={false}`.
 * The grid still runs its default generator for the lines it will not draw, and that
 * generator re-measures every X label the same way the axis does (154 of the forced
 * recalculations on the first open of the Pulls tab). Returning no points skips it.
 */
export function noVerticalGridLines(): number[] {
    return [];
}
