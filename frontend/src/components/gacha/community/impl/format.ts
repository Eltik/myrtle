export function fmtPct(frac: number, digits = 2): string {
    return `${(frac * 100).toFixed(digits)}%`;
}

/**
 * Compares an observed rate against a baseline (expected/advertised, or community avg)
 * using *percentage of baseline* - not point subtraction. A 6★ rate of 4% with an
 * expected of 2% reads as "200% of expected" (i.e. double), not "above by 2%".
 */
export interface IRateComparison {
    /** observed / baseline. 1.0 = on target. 0.5 = half. 2.0 = double. */
    ratio: number;
    /** "61% of expected", "247% of expected", "on target". */
    ratioLabel: string;
    /** "below by 39%", "above by 147%", "on target" - scaled to baseline. */
    deviationLabel: string;
    /** Numeric (ratio - 1) signed deviation from baseline. */
    deviation: number;
    /** "below" | "above" | "on" - for color coding. */
    direction: "below" | "above" | "on";
}

export function compareRate(observed: number, baseline: number): IRateComparison | null {
    if (!Number.isFinite(observed) || !Number.isFinite(baseline) || baseline <= 0) return null;
    const ratio = observed / baseline;
    const dev = ratio - 1;
    const onTarget = Math.abs(dev) < 0.01;
    const direction: "below" | "above" | "on" = onTarget ? "on" : dev > 0 ? "above" : "below";

    const ratioPct = ratio * 100;
    const ratioLabel = onTarget ? "on target" : ratioPct >= 999.5 ? `${(ratio).toFixed(1)}× baseline` : `${ratioPct.toFixed(0)}% of expected`;

    const devPct = Math.abs(dev * 100);
    const deviationLabel = onTarget ? "on target" : `${direction} by ${devPct < 10 ? devPct.toFixed(1) : devPct.toFixed(0)}%`;

    return { ratio, ratioLabel, deviationLabel, deviation: dev, direction };
}

export function fmtRelative(iso: string | null | undefined): string {
    if (!iso) return "-";
    const ts = Date.parse(iso);
    if (Number.isNaN(ts)) return "-";
    const seconds = Math.max(0, Math.floor((Date.now() - ts) / 1000));
    if (seconds < 60) return "moments ago";
    const minutes = Math.floor(seconds / 60);
    if (minutes < 60) return `${minutes}m ago`;
    const hours = Math.floor(minutes / 60);
    if (hours < 24) return `${hours}h ago`;
    const days = Math.floor(hours / 24);
    return `${days}d ago`;
}

export function fmtUTCStamp(iso: string | null | undefined): string {
    if (!iso) return "-";
    const d = new Date(iso);
    if (Number.isNaN(d.getTime())) return "-";
    return `${d.toISOString().slice(0, 16).replace("T", " ")} UTC`;
}
