/**
 * Landmarks on a swept damage curve, derived from the sampled points alone.
 *
 * Break point: the smallest swept DEF (or RES) from which the value stops falling,
 * i.e. every hit has reached the 5% floor. Between samples the true kink is unknown,
 * so the last falling segment's line is extended down to the floor value; damage is
 * linear in DEF above the floor, which makes that exact for a single hit type and an
 * estimate inside one sample step when several hit types floor at different DEF.
 *
 * Half point: where the value first falls to half of the value at the sweep's start,
 * interpolated linearly between the two samples that straddle it.
 */

export interface ICurveMarkerPoint {
    x: number;
    y: number;
}

export interface ICurveMarkers {
    breakPoint: ICurveMarkerPoint | null;
    halfPoint: ICurveMarkerPoint | null;
}

interface ISample {
    x: number;
    y: number;
}

/** Relative tolerance for "did not fall": guards float noise in the engine's output. */
const FLAT_EPSILON = 1e-9;

function flatTolerance(points: readonly ISample[]): number {
    let max = 0;
    for (const p of points) max = Math.max(max, Math.abs(p.y));
    return Math.max(FLAT_EPSILON, max * FLAT_EPSILON);
}

/**
 * Null when the curve never falls (true damage, or a RES sweep for a physical operator),
 * or when it is still falling at the last sample. The floor needs at least two samples
 * that do not fall to count as reached: one trailing sample cannot tell a floor from a
 * shallower slope.
 */
export function findBreakPoint(points: readonly ISample[]): ICurveMarkerPoint | null {
    const n = points.length;
    if (n < 3) return null;
    const eps = flatTolerance(points);
    // m = first index of the trailing run that never falls.
    let m = n - 1;
    while (m > 0 && points[m].y >= points[m - 1].y - eps) m--;
    if (m === 0 || m > n - 2) return null;
    const floor = points[m].y;
    const prev = points[m - 1];
    let x = points[m].x;
    if (m >= 2) {
        const before = points[m - 2];
        const slope = (before.y - prev.y) / (prev.x - before.x);
        if (slope > 0 && Number.isFinite(slope)) x = Math.min(points[m].x, Math.max(prev.x, prev.x + (prev.y - floor) / slope));
    }
    return { x, y: floor };
}

/** Null when the start value is not positive or the curve never reaches half of it in range. */
export function findHalfPoint(points: readonly ISample[]): ICurveMarkerPoint | null {
    if (points.length < 2) return null;
    const start = points[0].y;
    if (!(start > 0)) return null;
    const half = start / 2;
    for (let i = 1; i < points.length; i++) {
        const a = points[i - 1];
        const b = points[i];
        if (b.y > half) continue;
        const x = a.y === b.y ? b.x : a.x + ((a.y - half) / (a.y - b.y)) * (b.x - a.x);
        return { x, y: half };
    }
    return null;
}

/**
 * Markers for one series of chart rows (`rows[i][key]`). A series with any missing value
 * (still loading, or failed) gets none: a marker from a partial curve would be wrong.
 */
export function curveMarkersFor(rows: readonly { x: number; [key: string]: number | null }[], key: string): ICurveMarkers {
    const points: ISample[] = [];
    for (const row of rows) {
        const y = row[key];
        if (typeof y !== "number" || !Number.isFinite(y)) return { breakPoint: null, halfPoint: null };
        points.push({ x: row.x, y });
    }
    return { breakPoint: findBreakPoint(points), halfPoint: findHalfPoint(points) };
}
