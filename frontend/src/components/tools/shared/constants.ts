import type { ISweepRange } from "./types";

/** Shared chart line colors for DPS, HPS, and any future calculator tool. */
export const CHART_PALETTE: readonly string[] = ["oklch(0.65 0.20 30)", "oklch(0.62 0.18 220)", "oklch(0.62 0.18 145)", "oklch(0.62 0.18 305)", "oklch(0.70 0.16 65)", "oklch(0.60 0.20 0)", "oklch(0.62 0.16 190)", "oklch(0.55 0.20 270)"];

export function getInstanceColor(index: number): string {
    return CHART_PALETTE[index % CHART_PALETTE.length] ?? CHART_PALETTE[0];
}

export const STEP_OPTIONS: readonly number[] = [11, 21, 41];

/**
 * Build the X positions for a sweep. When `integer` is set (e.g. a target-count
 * axis) values span every whole number between min and max so a coarse step
 * count still yields one point per integer rather than fractional positions.
 */
export function buildSweepPoints(range: ISweepRange, integer = false): number[] {
    const steps = Math.max(2, Math.floor(range.steps));
    const min = Math.min(range.min, range.max);
    const max = Math.max(range.min, range.max);
    if (steps === 1 || min === max) return [integer ? Math.round(min) : min];

    if (integer) {
        const out: number[] = [];
        for (let v = Math.round(min); v <= Math.round(max); v++) out.push(v);
        return out.length > 0 ? out : [Math.round(min)];
    }

    const stride = (max - min) / (steps - 1);
    const out: number[] = [];
    for (let i = 0; i < steps; i++) {
        const value = min + stride * i;
        out.push(Math.round(value * 100) / 100);
    }
    return out;
}

export function formatLargeNumber(n: number | null | undefined): string {
    if (n == null || !Number.isFinite(n)) return "-";
    const v = Math.abs(n);
    if (v >= 1_000_000) return `${(n / 1_000_000).toFixed(2).replace(/\.?0+$/, "")}M`;
    if (v >= 10_000) return `${(n / 1_000).toFixed(1).replace(/\.0$/, "")}k`;
    if (v >= 1_000) return Math.round(n).toLocaleString("en-US");
    return (Math.round(n * 10) / 10).toString();
}
