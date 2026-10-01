/**
 * The story feature's numeric floor: the range clamp, and nothing else.
 *
 * FIVE modules had shipped their own (`settings.ts` a coerced slider, `palette.ts` a
 * lightness, `reading.ts` a words-per-minute and a daily budget, `chrome.ts` an idle wait and
 * a tooltip's left edge, `engine.ts` a private `clamp(n, max)` whose eleven call sites all
 * passed a zero lower bound). Each was correct and each had to be re-read to know it, so the
 * arithmetic lives here once and callers keep what the bounds MEAN.
 *
 * NaN passes straight through in both functions, like the hand-written
 * `Math.min(hi, Math.max(lo, n))`. Deliberate: a caller that can receive a non-number decides
 * what to do before clamping. `reading.ts` is the example: `Number(null)` is 0 and finite, so
 * it would clamp to the floor instead of falling back to the default.
 */

/** `n` held between `lo` and `hi` inclusive. */
export function clamp(n: number, lo: number, hi: number): number {
    return Math.min(hi, Math.max(lo, n));
}

/** `n` held to 0..1, the range alphas, volumes, fills and progress fractions all live in. */
export function clamp01(n: number): number {
    return Math.min(1, Math.max(0, n));
}
