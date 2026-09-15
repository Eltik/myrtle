import { DEFAULT_LOCALE, formatMessage, sourceMessage } from "#/lib/i18n";
import { fullMessageKey, type TypedT } from "#/lib/i18n/messages";
import type { messages as formatMessages } from "./format.messages";

/** The `t` these label helpers need, narrowed to the keys they can render. */
export type GachaFormatT = TypedT<typeof formatMessages>;

/**
 * Default `t` for a caller that is not inside an `I18nProvider` - today only
 * the stats page header, which reads `fmtRelative` from outside this feature.
 * It resolves against the bundled source catalog, so the English is the same
 * one the gacha components render and this file carries no second copy of it.
 */
const sourceT: GachaFormatT = (key, values) => formatMessage(sourceMessage(fullMessageKey("gacha", key)) ?? key, DEFAULT_LOCALE, values);

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

export function compareRate(observed: number, baseline: number, t: GachaFormatT = sourceT): IRateComparison | null {
    if (!Number.isFinite(observed) || !Number.isFinite(baseline) || baseline <= 0) return null;
    const ratio = observed / baseline;
    const dev = ratio - 1;
    const onTarget = Math.abs(dev) < 0.01;
    const direction: "below" | "above" | "on" = onTarget ? "on" : dev > 0 ? "above" : "below";

    const ratioPct = ratio * 100;
    const ratioLabel = onTarget ? t("community.compare.onTarget") : ratioPct >= 999.5 ? t("community.compare.ratioBaseline", { ratio: ratio.toFixed(1) }) : t("community.compare.ratioOfExpected", { percent: ratioPct.toFixed(0) });

    const devPct = Math.abs(dev * 100);
    const deviationLabel = onTarget ? t("community.compare.onTarget") : t("community.compare.deviation", { direction, percent: devPct < 10 ? devPct.toFixed(1) : devPct.toFixed(0) });

    return { ratio, ratioLabel, deviationLabel, deviation: dev, direction };
}

export function fmtRelative(iso: string | null | undefined, t: GachaFormatT = sourceT): string {
    if (!iso) return "-";
    const ts = Date.parse(iso);
    if (Number.isNaN(ts)) return "-";
    const seconds = Math.max(0, Math.floor((Date.now() - ts) / 1000));
    if (seconds < 60) return t("community.relative.moments");
    const minutes = Math.floor(seconds / 60);
    if (minutes < 60) return t("community.relative.minutes", { count: minutes });
    const hours = Math.floor(minutes / 60);
    if (hours < 24) return t("community.relative.hours", { count: hours });
    const days = Math.floor(hours / 24);
    return t("community.relative.days", { count: days });
}

export function fmtUTCStamp(iso: string | null | undefined): string {
    if (!iso) return "-";
    const d = new Date(iso);
    if (Number.isNaN(d.getTime())) return "-";
    return `${d.toISOString().slice(0, 16).replace("T", " ")} UTC`;
}
