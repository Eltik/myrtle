/**
 * The story feature's numeric floor: the range clamp, and nothing else.
 *
 * It exists because FIVE modules had shipped their own. `settings.ts` clamped a
 * coerced slider, `palette.ts` a lightness, `reading.ts` a words-per-minute and
 * a daily budget, `chrome.ts` an idle wait and a tooltip's left edge, and
 * `engine.ts` a private `clamp(n, max)` that every one of its eleven call sites
 * passed a zero lower bound to. Each was correct and each had to be re-read to
 * know it was, so the arithmetic lives here once and the callers keep only the
 * part that is theirs: what the bounds MEAN.
 *
 * NaN passes straight through, in both functions, exactly as the hand-written
 * `Math.min(hi, Math.max(lo, n))` did. That is deliberate: a caller that can
 * receive a non-number decides what to do about it before it clamps, and
 * `reading.ts` is the worked example, because `Number(null)` is 0 and finite
 * and would have clamped to the floor instead of falling back to the default.
 */

/** `n` held between `lo` and `hi` inclusive. */
export function clamp(n: number, lo: number, hi: number): number {
    return Math.min(hi, Math.max(lo, n));
}

/** `n` held to 0..1, the range alphas, volumes, fills and progress fractions all live in. */
export function clamp01(n: number): number {
    return Math.min(1, Math.max(0, n));
}
